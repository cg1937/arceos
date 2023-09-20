use alloc::string::{String, ToString};

#[repr(packed)]
#[derive(Debug)]
pub struct BootSector {
    pub bytes_per_sector: u16,       // 512/1024/2048/4096
    pub sectors_per_cluster: u8,     // 1/2/4/8/16/32/64/128
    pub reserved_sectors_count: u16, // 32 for fat32
    pub total_sectors32: u32,        // total sectors in the volume
    pub fat_count: u8,               // 2 for fat32
    pub sectors_per_fat32: u32,      // size of a FAT in unit of sector
    pub root_cluster: u32,           // first cluster of root directory, usually set to 2
    pub root_dir_sectors_count: u32, // size of root directory in unit of sector
    pub fsinfo_sector: u16,          // sector number of FSINFO structure, usually set to 1
    pub reserved: [u8; 488],         // fill to 512 bytes
}

impl BootSector {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const BootSector as *const u8,
                core::mem::size_of::<BootSector>(),
            )
        }
    }

    // pub fn new1

    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sectors_count as u32
    }

    pub fn bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector as u32
    }

    pub fn fat_sectors_count(&self) -> u32 {
        // self.sectors_per_fat32 * self.fat_count as u32
        self.sectors_per_fat32
    }

    pub fn root_dir_start_sector(&self) -> u32 {
        self.fat_start_sector() + self.fat_sectors_count() * 2
    }

    pub fn root_dir_sectors_count(&self) -> u32 {
        self.root_dir_sectors_count
    }

    pub fn data_start_sector(&self) -> u32 {
        self.root_dir_start_sector() // + self.root_dir_sectors_count()
    }

    pub fn data_sectors_count(&self) -> u32 {
        self.total_sectors32 - self.data_start_sector()
    }

    // return the number of clusters, <= 4085 is FAT12, >= 4086 and <= 65525 is FAT16, >= 65526 is FAT32
    pub fn clusters_count(&self) -> u32 {
        self.data_sectors_count() / self.sectors_per_cluster as u32
    }

    pub fn sector_to_cluster(&self, sector_id: u32) -> u32 {
        (sector_id - self.data_start_sector()) / self.sectors_per_cluster as u32 + 2
    }

    pub fn cluster_to_sector(&self, cluster_id: u32) -> u32 {
        self.data_start_sector() + (cluster_id - 2) * self.sectors_per_cluster as u32
    }
    pub fn bytes_per_cluster(&self) -> u32 {
        self.bytes_per_sector() * self.sectors_per_cluster as u32
    }
}

impl Default for BootSector {
    fn default() -> Self {
        Self {
            bytes_per_sector: 512,
            sectors_per_cluster: 1,
            reserved_sectors_count: 32,
            fat_count: 2,
            total_sectors32: 0,
            sectors_per_fat32: 0,
            root_cluster: 2,
            root_dir_sectors_count: 0,
            fsinfo_sector: 1,
            reserved: [0; 488],
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FSInfoSector {
    pub free_cluster_count: u32, // number of free clusters on the volume
    pub next_free_cluster: u32,  // cluster number of the first cluster searched for free cluster
    pub reserved: [u8; 504],
}

impl FSInfoSector {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const FSInfoSector as *const u8,
                core::mem::size_of::<FSInfoSector>(),
            )
        }
    }

    pub fn new(free_cluster_count: u32, next_free_cluster: u32) -> Self {
        Self {
            free_cluster_count,
            next_free_cluster,
            reserved: [0; 504],
        }
    }
}

impl Default for FSInfoSector {
    fn default() -> Self {
        Self {
            free_cluster_count: 0,
            next_free_cluster: 0,
            reserved: [0; 504],
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct DirEntryAttr: u8 {
        const ReadOnly = 0x01;
        const Hidden = 0x02;
        const System = 0x04;
        const VolumeId = 0x08;
        const Directory = 0x10;
        const Archive = 0x20;
    }
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct DirEntry {
    pub name: [u8; 23],     // SFN(Short file name) of the object
    pub attr: DirEntryAttr, // file attribute
    pub first_cluster: u32, // high word of this entry's first cluster number (always 0 for a FAT12 or FAT16 volume)
    pub file_size: u32,     // 32-bit DWORD holding this file's size in bytes
}

impl DirEntry {
    // implement a new function for DirEntry, function's parameter is a slice of 32 bytes, then return a packed DirEntry struct
    pub fn new(buf: &[u8]) -> Self {
        let mut dir_entry = Self::default();
        dir_entry.name.copy_from_slice(&buf[0..23]);
        dir_entry.attr = match buf[23] {
            0x01 => DirEntryAttr::ReadOnly,
            0x02 => DirEntryAttr::Hidden,
            0x04 => DirEntryAttr::System,
            0x08 => DirEntryAttr::VolumeId,
            0x10 => DirEntryAttr::Directory,
            0x20 => DirEntryAttr::Archive,
            _ => DirEntryAttr::Archive,
        };
        dir_entry.first_cluster = u32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]);
        dir_entry.file_size = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
        dir_entry
    }

    pub fn increase_file_size(&mut self, size: u32) {
        self.file_size += size;
    }

    pub fn decrease_file_size(&mut self, size: u32) {
        self.file_size -= size;
    }

    pub fn is_valid(&self) -> bool {
        self.name[0] != 0xE5 && self.name[0] != 0x00
    }

    pub fn is_dir(&self) -> bool {
        self.attr.contains(DirEntryAttr::Directory)
    }

    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }

    pub fn name(&self) -> Option<String> {
        let name_string = match core::str::from_utf8(&self.name) {
            Ok(utf8_str) => utf8_str.trim_matches('\0').to_string(),
            Err(_) => {
                return None;
            }
        };
        // check if the name string is ascii and if this is a dir, can only use letter and numbers
        if self.is_dir()
            && !name_string
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            None
        } else {
            Some(name_string)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 32) }
    }
}

impl Default for DirEntryAttr {
    fn default() -> Self {
        Self::Archive
    }
}

impl Default for DirEntry {
    fn default() -> Self {
        Self {
            name: [0u8; 23],
            attr: DirEntryAttr::Archive,
            first_cluster: 0,
            file_size: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum FatMarker {
    Free,
    Reserved,
    InUse(u32), // 包含下一个索引的值
    BadCluster,
    EndOfChain,
}

impl FatMarker {
    pub fn from_value(value: u32) -> Self {
        match value {
            0x00000000 => FatMarker::Free,
            0x00000001 => FatMarker::Reserved,
            0x0FFFFFF7 => FatMarker::BadCluster,
            0x0FFFFFF8..=0x0FFFFFFF => FatMarker::EndOfChain,
            _ => FatMarker::InUse(value),
        }
    }
}

pub fn convert_to_u8_array(s: &str) -> Option<[u8; 23]> {
    let bytes = s.as_bytes();
    if bytes.len() > 23 {
        return None;
    }
    let mut array = [0u8; 23];
    array[..bytes.len()].copy_from_slice(bytes);
    Some(array)
}
