use alloc::string::ToString;
use alloc::vec::Vec;
use axdriver::prelude::*;

use alloc::sync::Arc;
use core::slice::from_raw_parts;
use spin::RwLock;

use crate::dir::{Dir, DirNode};
use crate::layout::{BootSector, DirEntry, FSInfoSector, FatMarker};
use crate::sector::SectorManager;
use axfs_vfs::{VfsNodeRef, VfsOps};

/// A abstraction Struct of a FAT32 filesystem.
pub struct CCFileSystem {
    root: RwLock<Option<Arc<DirNode>>>,
    sector_manager: RwLock<SectorManager>,
    boot_sector: RwLock<BootSector>,
    fs_info_sector: RwLock<FSInfoSector>,
    fat: RwLock<Vec<u32>>,
}

impl CCFileSystem {
    /// Create a new instance of CCFileSystem.
    pub fn new(sector_manager: Option<SectorManager>) -> Self {
        Self {
            root: RwLock::new(None),
            sector_manager: RwLock::new(sector_manager.unwrap()),
            boot_sector: RwLock::new(BootSector::default()),
            fs_info_sector: RwLock::new(FSInfoSector::default()),
            fat: RwLock::new(Vec::new()),
        }
    }

    /// Return the bytes per cluster.
    pub fn bytes_per_cluster(&self) -> u32 {
        self.boot_sector.read().bytes_per_cluster()
    }

    /// Init the filesystem by reading boot sector and sector manager, then initialize the root dir and the fat table.
    pub fn init(&self) -> Result<(), DevError> {
        self.sector_manager.write().set_position(0);
        let mut boot_sector = self.boot_sector.write();
        *boot_sector = BootSector {
            bytes_per_sector: self
                .sector_manager
                .read()
                .read_16_seq()
                .expect("read sector manager bytes_per_sector failed"),
            sectors_per_cluster: self
                .sector_manager
                .read()
                .read_8_seq()
                .expect("read sector manager sectors_per_cluster failed"),
            reserved_sectors_count: self
                .sector_manager
                .read()
                .read_16_seq()
                .expect("read sector manager reserved_sectors_count failed"),
            total_sectors32: self
                .sector_manager
                .read()
                .read_32_seq()
                .expect("read sector manager total_sectors32 failed"),
            fat_count: self
                .sector_manager
                .read()
                .read_8_seq()
                .expect("read sector manager fat_count failed"),
            sectors_per_fat32: self
                .sector_manager
                .read()
                .read_32_seq()
                .expect("read sector manager sectors_per_fat32 failed"),
            root_cluster: self
                .sector_manager
                .read()
                .read_32_seq()
                .expect("read sector manager root_cluster failed"),
            root_dir_sectors_count: self
                .sector_manager
                .read()
                .read_32_seq()
                .expect("read sector manager root_dir_sectors_count failed"),
            fsinfo_sector: self
                .sector_manager
                .read()
                .read_16_seq()
                .expect("read sector manager fs info sector failed"),
            reserved: [0u8; 488],
        };
        drop(boot_sector);
        self.sector_manager.write().set_position(512);
        let mut fs_info_sector = self.fs_info_sector.write();
        *fs_info_sector = FSInfoSector {
            free_cluster_count: self.sector_manager.read().read_32_seq()?,
            next_free_cluster: self.sector_manager.read().read_32_seq()?,
            reserved: [0u8; 504],
        };
        drop(fs_info_sector);
        let root_dir_start_sector = self.boot_sector.read().root_dir_start_sector() as u64 * 512;
        self.sector_manager
            .write()
            .set_position(root_dir_start_sector);
        self.init_fat_table().expect("read fat table failed");
        self.init_root().expect("init root failed");
        Ok(())
    }

    /// Return the next free cluster.
    pub fn get_next_free_cluster(&self) -> u32 {
        self.fs_info_sector.read().next_free_cluster
    }

    /// Init the root directory.
    fn init_root(&self) -> DevResult<()> {
        let root_dir_start_sector = self.boot_sector.read().root_dir_start_sector() as u64;
        let root_dir_sector_count = self.boot_sector.read().root_dir_sectors_count() as u64;
        let root_first_cluster = self
            .boot_sector
            .read()
            .sector_to_cluster(root_dir_start_sector as u32);

        let root_last_cluster = self
            .boot_sector
            .read()
            .sector_to_cluster(root_dir_start_sector as u32 + root_dir_sector_count as u32 - 1);

        let mut fat = self.fat.write();

        for cluster in root_first_cluster..=root_last_cluster {
            fat[cluster as usize] = if cluster == root_last_cluster {
                0xFFFFFFFF
            } else {
                (cluster + 1) as u32
            };
        }
        drop(fat);
        self.sector_manager
            .write()
            .set_position(root_dir_start_sector);
        let mut clusters = Vec::new();
        for _ in 0..root_dir_sector_count {
            clusters.append(&mut self.sector_manager.read().read_sector_seq()?);
        }

        let mut new_entries = Vec::new();
        for i in 0..clusters.len() / 32 {
            let entry = DirEntry::new(&clusters[i * 32..(i + 1) * 32]);
            if !entry.is_valid() {
                break;
            }
            new_entries.push(entry);
        }
        let root_dir_node = DirNode::new(Dir::new_root(&new_entries), "/".to_string(), None);
        root_dir_node
            .update_children()
            .expect("update chilren failed");
        let mut root = self.root.write();
        *root = Some(root_dir_node);
        Ok(())
    }

    /// Read FAT area from disk and init the fat table.
    fn init_fat_table(&self) -> DevResult<()> {
        let fat_sectors_count = self.boot_sector.read().fat_sectors_count() as u64;
        let fat_start_sector = self.boot_sector.read().fat_start_sector() as u64;
        let sector_size = self.sector_manager.read().sector_size() as u64;
        let mut fat_data = Vec::new();
        self.sector_manager
            .write()
            .set_position(fat_start_sector * sector_size);
        for _ in fat_start_sector..(fat_start_sector + fat_sectors_count) {
            let sector_data = self.sector_manager.read().read_sector_seq()?;
            fat_data.extend_from_slice(&sector_data);
        }
        let fat_entries_count =
            fat_sectors_count * (self.boot_sector.read().bytes_per_sector() as u64 / 4);
        self.fat.write().resize(fat_entries_count as usize, 0);
        self.fat.write().copy_from_slice(unsafe {
            from_raw_parts(
                fat_data.as_ptr() as *const u32,
                fat_data.len() / core::mem::size_of::<u32>(),
            )
        });
        Ok(())
    }

    /// Returns the reference of root directory node.
    pub fn root_dir_node(&self) -> Option<Arc<DirNode>> {
        self.root.read().clone()
    }

    /// Return the fat entry via cluster_id.
    pub fn get_fat_entry(&self, cluster_id: u32) -> Result<u32, DevError> {
        let fat = self.fat.read();
        if cluster_id >= fat.len() as u32 || cluster_id < 2u32 {
            return Err(DevError::Unsupported);
        }
        Ok(fat[cluster_id as usize])
    }

    /// Read a cluster from disk via cluster_id.
    pub fn read_cluster(&self, cluster_id: u32) -> Result<Vec<u8>, DevError> {
        let cluster_start_sector =
            self.boot_sector.read().cluster_to_sector(cluster_id) as u64 * 512;
        let mut cluster = Vec::new();
        self.sector_manager
            .write()
            .set_position(cluster_start_sector);
        for _ in 0..self.boot_sector.read().sectors_per_cluster as u64 {
            cluster.append(&mut self.sector_manager.read().read_sector_seq()?);
        }
        Ok(cluster)
    }

    /// Return true if the cluster is the end of the chain.
    pub fn is_end(&self, index: u32) -> bool {
        FatMarker::from_value(index) == FatMarker::EndOfChain
    }

    /// Return true if the cluster is bad.
    pub fn is_bad_cluster(&self, index: u32) -> bool {
        FatMarker::from_value(index) == FatMarker::BadCluster
    }

    /// Write a cluster to disk via cluster_id.
    pub fn write_cluster(&self, cluster_id: u32, data: &[u8]) -> Result<(), DevError> {
        let cluster_start_sector =
            self.boot_sector.read().cluster_to_sector(cluster_id) as u64 * 512;
        self.sector_manager
            .write()
            .set_position(cluster_start_sector);
        for i in 0..self.boot_sector.read().sectors_per_cluster as u64 {
            self.sector_manager
                .write()
                .write_sector_seq(&data[i as usize * 512..(i + 1) as usize * 512])?;
        }
        Ok(())
    }

    /// Update free cluster count via calculating the number of free clusters of the fat.
    fn update_free_cluster_count(&self) {
        let count = self
            .fat
            .read()
            .iter()
            .skip(2)
            .filter(|&&entry| entry == 0x00000000)
            .count();
        self.fs_info_sector.write().free_cluster_count = count as u32;
    }

    /// Find the next free cluster.
    fn find_next_free_cluster(&self) -> Option<u32> {
        if self.fs_info_sector.read().free_cluster_count == 0 {
            // warn!("No free cluster");
            return None;
        }
        let next_free_cluster = self.fs_info_sector.read().next_free_cluster;
        let fat = self.fat.read();
        if next_free_cluster < fat.len() as u32 && fat[next_free_cluster as usize] == 0x00000000 {
            return Some(next_free_cluster);
        }
        None
    }

    // Update the next free cluster.
    fn update_next_free_cluster(&self) -> Result<(), DevError> {
        if let Some((index, _)) = self
            .fat
            .read()
            .iter()
            .skip(2)
            .enumerate()
            .find(|&(_, &entry)| entry == 0x00000000)
        {
            let mut fs_info_sector = self.fs_info_sector.write();
            fs_info_sector.next_free_cluster = index as u32 + 2u32;
            Ok(())
        } else {
            Err(DevError::Unsupported)
        }
    }

    /// Write back fs_info_sector of memory to disk.
    pub fn flush_fs_info_sector(&self) -> Result<(), DevError> {
        self.sector_manager.write().set_position(512);
        self.update_free_cluster_count();
        self.sector_manager
            .write()
            .write_32_seq(self.fs_info_sector.read().free_cluster_count)?;
        self.sector_manager
            .write()
            .write_32_seq(self.fs_info_sector.read().next_free_cluster)?;
        Ok(())
    }

    /// Allocate a cluster between curr_cluster_id and next_cluster_id.
    pub fn allocate_cluster_at_middle(
        &self,
        curr_cluster_id: u32,
        next_cluster_id: u32,
    ) -> Option<u32> {
        let next_free_cluster = self.find_next_free_cluster()?;
        let mut fat = self.fat.write();
        fat[next_free_cluster as usize] = next_cluster_id;
        fat[curr_cluster_id as usize] = next_free_cluster;
        drop(fat);
        self.fs_info_sector.write().free_cluster_count -= 1;
        self.update_next_free_cluster().ok()?;
        Some(next_free_cluster)
    }

    /// Allocate a cluster at the end of the chain.
    pub fn allocate_cluster_at_end(&self, curr_cluster_id: u32) -> Option<u32> {
        let next_free_cluster = self.find_next_free_cluster()?;
        let mut fat = self.fat.write();
        fat[next_free_cluster as usize] = 0x0FFFFFFF;
        fat[curr_cluster_id as usize] = next_free_cluster;
        drop(fat);
        self.fs_info_sector.write().free_cluster_count -= 1;
        self.update_next_free_cluster().ok()?;
        Some(next_free_cluster)
    }

    /// Allocate a cluster at the start of the chain.
    pub fn allocate_cluster_at_start(&self) -> Option<u32> {
        let next_free_cluster = self.find_next_free_cluster()?;
        let mut fat = self.fat.write();
        fat[next_free_cluster as usize] = 0x0FFFFFFF;
        self.fs_info_sector.write().free_cluster_count -= 1;
        drop(fat);
        self.update_next_free_cluster().ok()?;
        Some(next_free_cluster)
    }

    /// Link the cluster to the end of the chain.
    pub fn link_to_end(&self, curr_cluster_id: u32) -> Result<(), DevError> {
        let mut fat = self.fat.write();
        fat[curr_cluster_id as usize] = 0x0FFFFFFF;
        drop(fat);
        self.update_next_free_cluster()?;
        Ok(())
    }

    /// Free the cluster, then update the free_cluster_count and next_free_cluster.
    pub fn free_cluster(&self, cluster_id: u32) -> Result<(), DevError> {
        let mut fat = self.fat.write();
        fat[cluster_id as usize] = 0x00000000;
        drop(fat);
        self.fs_info_sector.write().free_cluster_count += 1;
        self.fs_info_sector.write().next_free_cluster = cluster_id;
        // self.flush_fs_info_sector()?;
        Ok(())
    }
}

impl VfsOps for CCFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        self.root_dir_node().unwrap().clone()
    }
}
