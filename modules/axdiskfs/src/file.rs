use crate::dir::DirNode;
use crate::disk::SeekFrom;
use crate::FS;

use alloc::sync::{Arc, Weak};

use crate::alloc::string::ToString;
use alloc::string::String;

use alloc::vec::Vec;
use axdriver::prelude::*;
use axfs_vfs::{
    impl_vfs_non_dir_default, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType,
    VfsResult,
};
use driver_block::DevResult;
use spin::RwLock;

use core::cmp::max;

#[derive(Clone)]
pub struct File {
    size: u32,
    first_cluster: u32,   // 文件的第一个簇号
    current_cluster: u32, // 当前读取的簇号
    offset: u32,          // 当前读取的簇的偏移量
}

impl File {
    pub fn new(first_cluster: u32) -> Self {
        Self {
            size: 0,
            first_cluster,
            current_cluster: first_cluster,
            offset: 0,
        }
    }

    fn size(&self) -> u64 {
        self.size as u64
    }

    fn position(&self) -> u64 {
        let fs_mutex = FS.try_get().expect("fs is not initialized");
        self.current_cluster as u64 * fs_mutex.bytes_per_cluster() as u64 + self.offset as u64
    }

    pub fn read_all(&mut self) -> DevResult<Vec<u8>> {
        let mut data = Vec::new();
        let mut all_clusters = Vec::new();
        let mut curr_cluster = self.first_cluster;

        let fs_arc = FS.try_get().expect("fs is not initialized");
        // 遍历 FAT 表获取文件的所有簇
        while !fs_arc.is_end(curr_cluster) {
            // if curr_cluster is Bad cluster
            if fs_arc.is_bad_cluster(curr_cluster) {
                return Err(DevError::Unsupported);
            }
            all_clusters.push(curr_cluster);
            curr_cluster = fs_arc.get_fat_entry(curr_cluster)?;
        }

        // 逐个簇读取数据并追加到缓冲区
        for cluster_id in all_clusters {
            let cluster_data = fs_arc.read_cluster(cluster_id)?;
            data.extend_from_slice(&cluster_data);
        }

        // truncate the data via file size
        data.truncate(self.size() as usize);

        Ok(data)
    }

    pub fn read_at(&mut self, byte_offset: u64, buf: &mut [u8]) -> DevResult<usize> {
        if byte_offset >= self.size() {
            return Ok(0usize);
        }

        let fs_arc = FS.try_get().expect("fs is not initialized");
        // set the current_cluster and offset
        let file_size = self.size();
        let mut offset = byte_offset % fs_arc.bytes_per_cluster() as u64;

        // calcaute the clusters'numbers  via byte_offset
        let clusters_num = byte_offset / fs_arc.bytes_per_cluster() as u64;

        // let the current cluster locate at the byte_offset position
        let mut cluster = self.first_cluster;

        for _ in 0..clusters_num {
            cluster = fs_arc.get_fat_entry(cluster)?;
        }

        // read the data from the current cluster, util the buf is full
        let mut buf_offset = 0;
        let mut prev_cluster = cluster;
        while buf_offset < buf.len() && !fs_arc.is_end(cluster) {
            let cluster_data = fs_arc.read_cluster(cluster)?;
            let remaining_data = file_size - (byte_offset + buf_offset as u64);
            let remaining_space_in_buf = buf.len() - buf_offset;
            let read_size = remaining_data
                .min(fs_arc.bytes_per_cluster() as u64 - offset)
                .min(remaining_space_in_buf as u64) as usize;
            buf[buf_offset..buf_offset + read_size]
                .copy_from_slice(&cluster_data[offset as usize..offset as usize + read_size]);
            buf_offset += read_size;

            if read_size == fs_arc.bytes_per_cluster() as usize - offset as usize {
                offset = 0;
            } else {
                offset += read_size as u64;
            }
            prev_cluster = cluster;
            cluster = fs_arc.get_fat_entry(cluster)?;
        }

        // finally update file's curr_cluster and offset
        self.current_cluster = prev_cluster;
        self.offset = offset as u32;

        // return the read size
        Ok(buf_offset)
    }

    // read_seq: use curr_cluster and offset to read data(read to the current cluster end)
    pub fn read_seq(&mut self) -> DevResult<Vec<u8>> {
        let mut data = Vec::new();
        let fs_mutex = FS.try_get().expect("fs is not initialized");
        let cluster_data = fs_mutex.read_cluster(self.current_cluster)?;
        let read_size = fs_mutex.bytes_per_cluster() as usize - self.offset as usize;
        data.extend_from_slice(
            &cluster_data[self.offset as usize..self.offset as usize + read_size as usize],
        );
        self.offset = 0;
        self.current_cluster = fs_mutex.get_fat_entry(self.current_cluster)?;
        Ok(data)
    }

    pub fn write_at(&mut self, byte_offset: u64, buf: &[u8]) -> DevResult<usize> {
        // can't write data beyond the file size
        if byte_offset > self.size() || (self.size() == 0 && byte_offset != 0) {
            return Err(DevError::Unsupported);
        }

        let fs_arc = FS.try_get().expect("fs is not initialized");
        // set the current_cluster and offset
        let mut offset = byte_offset % fs_arc.bytes_per_cluster() as u64;
        let old_size = self.size();

        // calcaute the clusters'numbers  via byte_offset
        let clusters_num = byte_offset / fs_arc.bytes_per_cluster() as u64;

        // let the current cluster locate at the byte_offset position
        let mut cluster = self.first_cluster;

        for _ in 0..clusters_num {
            cluster = fs_arc.get_fat_entry(cluster)?;
        }
        // write the data to the current cluster, util the buf is full
        let mut buf_offset = 0;
        while buf_offset < buf.len() {
            // if cluster is end of file, need to allocate a new cluster
            let mut cluster_data = fs_arc.read_cluster(cluster)?;
            let remaining_space = fs_arc.bytes_per_cluster() as usize - offset as usize;
            let write_size = (buf.len() - buf_offset).min(remaining_space as usize);
            cluster_data[offset as usize..(offset as usize + write_size as usize) as usize]
                .copy_from_slice(&buf[buf_offset..buf_offset + write_size]);
            buf_offset += write_size;
            offset += write_size as u64;
            fs_arc.write_cluster(cluster, &cluster_data)?;
            let next_cluster = fs_arc.get_fat_entry(cluster)?;
            if buf_offset < buf.len() {
                if fs_arc.is_end(next_cluster) {
                    cluster = fs_arc
                        .allocate_cluster_at_end(cluster)
                        .ok_or(DevError::Unsupported)?;
                } else {
                    cluster = next_cluster;
                }
                offset = 0;
            }
        }

        // finally update file's curr_cluster and offset
        self.current_cluster = cluster;
        self.offset = offset as u32;

        // calculate the size of the file after writing
        self.update_file_size(max(old_size, byte_offset + buf.len() as u64) as u32);

        // return the read size
        Ok(buf_offset)
    }

    pub fn write_seq(&mut self, buf: &[u8]) -> DevResult<usize> {
        let fs_arc = FS.try_get().expect("fs is not initialized");

        // Step 1: Get the current cluster and offset
        let mut cluster = self.current_cluster;
        let mut offset = self.offset as u64;

        let old_size = self.size();

        // Step 3: Calculate the remaining space in the current cluster
        let mut buf_offset = 0;
        while buf_offset < buf.len() {
            // Step 4: Start a loop until all data is written
            let mut cluster_data = fs_arc.read_cluster(cluster)?;
            let remaining_space = fs_arc.bytes_per_cluster() as usize - offset as usize;
            let write_size = (buf.len() - buf_offset).min(remaining_space as usize);
            cluster_data[offset as usize..offset as usize + write_size]
                .copy_from_slice(&buf[buf_offset..buf_offset + write_size]);
            buf_offset += write_size;
            offset += write_size as u64;
            fs_arc.write_cluster(cluster, &cluster_data)?;

            let next_cluster = fs_arc.get_fat_entry(cluster)?;
            if buf_offset < buf.len() {
                if fs_arc.is_end(next_cluster) {
                    cluster = fs_arc
                        .allocate_cluster_at_end(cluster)
                        .ok_or(DevError::Unsupported)?;
                } else {
                    cluster = next_cluster;
                }
                offset = 0;
            }
        }

        // Step 5: Update the file's current cluster and offset
        self.current_cluster = cluster;
        self.offset = offset as u32;

        // Step 6: Calculate the size of the file after writing
        self.update_file_size(max(old_size, self.offset as u64 + buf.len() as u64) as u32);

        // Step 7: Return the written size
        Ok(buf_offset)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<(), DevError> {
        match pos {
            SeekFrom::Start(pos) => {
                self.seek_from_start(pos)?;
            }
            SeekFrom::Current(off) => {
                self.seek_from_current(off)?;
            }
            SeekFrom::End(off) => {
                self.seek_from_end(off)?;
            }
        }
        Ok(())
    }

    fn seek_from_start(&mut self, pos: u64) -> Result<(), DevError> {
        if pos >= self.size() {
            return Err(DevError::Unsupported);
        }

        let fs_mutex = FS.try_get().expect("fs is not initialized");
        // first calucate the clusters'numbers via pos
        let clusters_num = pos / fs_mutex.bytes_per_cluster() as u64;
        // use cluster_num to reach the current cluster
        let mut curr_cluster = self.first_cluster;
        for _ in 0..clusters_num {
            curr_cluster = fs_mutex.get_fat_entry(curr_cluster)?;
        }
        // finally update file's curr_cluster and offset
        self.current_cluster = curr_cluster;
        self.offset = pos as u32 % fs_mutex.bytes_per_cluster() as u32;
        Ok(())
    }

    fn seek_from_end(&mut self, off: i64) -> Result<(), DevError> {
        let size = self.size();
        let new_pos = size.checked_add_signed(off).ok_or(DevError::Unsupported)?;
        self.seek_from_start(new_pos)
    }

    //TODO: if off is positive, not need use seek_from_start
    fn seek_from_current(&mut self, off: i64) -> Result<(), DevError> {
        let new_pos = self
            .position()
            .checked_add_signed(off)
            .ok_or(DevError::Unsupported)?;
        self.seek_from_start(new_pos)
    }

    pub fn truncate(&mut self, size: u64) -> Result<(), DevError> {
        // size should be cluster's multiple
        let fs_arc = FS.try_get().expect("fs is not initialized");

        let current_size = self.size() as u64;
        let cluster_size = fs_arc.bytes_per_cluster() as u64;

        if size == current_size {
            return Ok(());
        }

        // if size < file size, free the superfluous clusters, and update the file size
        if size < self.size() {
            let new_cluster_count = (size + cluster_size - 1) / cluster_size;
            let mut curr_cluster = self.first_cluster;
            let mut prev_cluster = curr_cluster;
            for _ in 0..new_cluster_count {
                prev_cluster = curr_cluster;
                curr_cluster = fs_arc.get_fat_entry(curr_cluster)?;
            }
            let mut cluster_to_free = curr_cluster;

            while !fs_arc.is_end(cluster_to_free) {
                let next_cluster = fs_arc.get_fat_entry(cluster_to_free)?;

                fs_arc.free_cluster(cluster_to_free)?;
                if fs_arc.is_end(next_cluster) {
                    break;
                }
                cluster_to_free = next_cluster;
            }

            fs_arc.link_to_end(prev_cluster)?;
        } else {
            let additional_clusters = (size - current_size + cluster_size - 1) / cluster_size;
            let mut last_cluster = self.first_cluster;
            while !fs_arc.is_end(last_cluster) {
                last_cluster = fs_arc.get_fat_entry(last_cluster)?;
            }
            for _ in 0..additional_clusters {
                last_cluster = fs_arc
                    .allocate_cluster_at_end(last_cluster)
                    .ok_or(DevError::Unsupported)?;
            }
        }
        self.update_file_size(size as u32);
        self.current_cluster = self.first_cluster;
        self.offset = 0;
        Ok(())
    }

    pub fn update_file_size(&mut self, size: u32) {
        self.size = size;
    }
}

pub struct FileNode {
    file: RwLock<File>,
    name: RwLock<String>,
    parent: RwLock<Weak<DirNode>>,
}

impl Clone for FileNode {
    fn clone(&self) -> Self {
        FileNode {
            file: RwLock::new(self.file.read().clone()),
            name: RwLock::new(self.name.read().clone()),
            parent: RwLock::new(self.parent.read().clone()),
        }
    }
}

impl FileNode {
    pub fn new(file: File, name: String, parent: Option<Weak<DirNode>>) -> Self {
        Self {
            file: RwLock::new(file),
            name: RwLock::new(name),
            // parent: Arc::downgrade(&parent.unwrap()),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<DirNode>::new())),
        }
    }

    pub fn self_rename(&self, new_name: &str) {
        let mut name_lock = self.name.write();
        *name_lock = new_name.to_string();
    }

    pub fn get_name(&self) -> String {
        self.name.read().clone()
    }

    pub fn get_size(&self) -> u64 {
        self.file.read().size() as u64
    }

    pub fn parent(&self) -> Option<Arc<DirNode>> {
        self.parent.read().upgrade()
    }

    pub fn update_size(&self) -> Result<(), DevError> {
        let new_size = self.file.read().size() as u32;
        let parent = self.parent.write().upgrade().unwrap();
        parent.update_child_file_size(self.name.read().clone().as_str(), new_size)
    }

    pub fn set_parent(&self, parent: Weak<DirNode>) {
        let mut parent_lock = self.parent.write();
        *parent_lock = parent;
    }

    pub fn read_at_test(&self, byte_offset: u64, buf: &mut [u8]) -> Result<usize, DevError> {
        self.file.write().read_at(byte_offset, buf)
    }

    pub fn write_at_test(&self, byte_offset: u64, buf: &[u8]) -> Result<usize, DevError> {
        let res = self.file.write().write_at(byte_offset, buf);

        self.update_size()?;
        res
    }

    pub fn read_all_test(&self) -> Result<Vec<u8>, DevError> {
        self.file.write().read_all()
    }

    pub fn truncate_test(&self, size: u64) -> Result<(), DevError> {
        let res = self.file.write().truncate(size);
        self.update_size()?;
        res
    }

    pub fn is_empty(&self) -> bool {
        self.file.read().size() == 0
    }
}

impl VfsNodeOps for FileNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let file_size = self.file.read().size();
        // 一个块的大小为 512 字节
        let blocks = file_size / 512 + if file_size % 512 == 0 { 0 } else { 1 };
        Ok(VfsNodeAttr::new(
            VfsNodePerm::from_bits_truncate(0o755),
            VfsNodeType::File,
            file_size,
            blocks,
        ))
    }

    fn truncate(&self, size: u64) -> VfsResult {
        self.truncate_test(size).map_err(|e| match e {
            _ => VfsError::Unsupported,
        })
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.read_at_test(offset, buf).map_err(|e| match e {
            _ => VfsError::Unsupported,
        })
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.write_at_test(offset, buf).map_err(|e| match e {
            _ => VfsError::Unsupported,
        })
    }

    impl_vfs_non_dir_default! {}
}
