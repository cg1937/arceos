use alloc::string::{String, ToString};

/// A Struct representing the boot sector of a FAT32 volume
#[repr(packed)]
#[derive(Debug)]
pub struct BootSector {
    /// bytes per sector, usually set to 512
    pub bytes_per_sector: u16,
    /// sectors per cluster, usually set to 1
    pub sectors_per_cluster: u8,
    /// reserved sectors count, usually set to 32 for fat32
    pub reserved_sectors_count: u16,
    /// total sectors in the volume
    pub total_sectors32: u32,
    /// number of FATs, usually set to 2
    pub fat_count: u8,
    /// size of a FAT in unit of sector
    pub sectors_per_fat32: u32,
    /// first cluster of root directory, usually set to 2
    pub root_cluster: u32,
    /// size of root directory in unit of sector
    pub root_dir_sectors_count: u32,
    /// sector number of FSINFO structure, usually set to 1
    pub fsinfo_sector: u16,
    /// reserved, fill to 512 bytes
    pub reserved: [u8; 488],
}

impl BootSector {
    /// convert the BootSector struct to a slice of bytes
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const BootSector as *const u8,
                core::mem::size_of::<BootSector>(),
            )
        }
    }

    /// fat_start_sector: the start sector of FAT
    pub fn fat_start_sector(&self) -> u32 {
        self.reserved_sectors_count as u32
    }

    /// bytes_per_sector: the bytes per sector
    pub fn bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector as u32
    }

    /// fat_sectors_count: the number of sectors in FAT
    pub fn fat_sectors_count(&self) -> u32 {
        // self.sectors_per_fat32 * self.fat_count as u32
        self.sectors_per_fat32
    }

    /// root_dir_start_sector: the start sector of root directory
    pub fn root_dir_start_sector(&self) -> u32 {
        self.fat_start_sector() + self.fat_sectors_count() * 2
    }

    /// root_dir_sectors_count: the number of sectors in root directory
    pub fn root_dir_sectors_count(&self) -> u32 {
        self.root_dir_sectors_count
    }

    /// data_start_sector: the start sector of data area
    pub fn data_start_sector(&self) -> u32 {
        self.root_dir_start_sector() // + self.root_dir_sectors_count()
    }

    /// data_sectors_count: the number of sectors in data area
    pub fn data_sectors_count(&self) -> u32 {
        self.total_sectors32 - self.data_start_sector()
    }

    /// return the number of clusters, <= 4085 is FAT12, >= 4086 and <= 65525 is FAT16, >= 65526 is FAT32
    pub fn clusters_count(&self) -> u32 {
        self.data_sectors_count() / self.sectors_per_cluster as u32
    }

    /// cluster_to_sector: convert cluster id to sector id
    pub fn sector_to_cluster(&self, sector_id: u32) -> u32 {
        (sector_id - self.data_start_sector()) / self.sectors_per_cluster as u32 + 2
    }

    /// sector_to_cluster: convert sector id to cluster id
    pub fn cluster_to_sector(&self, cluster_id: u32) -> u32 {
        self.data_start_sector() + (cluster_id - 2) * self.sectors_per_cluster as u32
    }

    /// bytes_per_cluster: the bytes per cluster
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

/// A Struct representing the FSInfo sector of a FAT32 volume
#[repr(C)]
#[derive(Debug)]
pub struct FSInfoSector {
    /// number of free clusters on the volume
    pub free_cluster_count: u32,
    /// cluster number of the first cluster searched for free cluster
    pub next_free_cluster: u32,
    /// reserved, fill to 512 bytes
    pub reserved: [u8; 504],
}

impl FSInfoSector {
    /// convert the FSInfoSector struct to a slice of bytes
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const FSInfoSector as *const u8,
                core::mem::size_of::<FSInfoSector>(),
            )
        }
    }

    /// new a FSInfoSector struct
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
    /// A Struct representing the attribute of a directory entry
    #[derive(Debug, Copy, Clone)]
    pub struct DirEntryAttr: u8 {
        /// read only
        const ReadOnly = 0x01;
        /// hidden
        const Hidden = 0x02;
        /// system
        const System = 0x04;
        /// volume id
        const VolumeId = 0x08;
        /// if Directory is set, this is a directory, else this is a file
        const Directory = 0x10;
        /// archive
        const Archive = 0x20;
    }
}

/// A Struct representing a directory entry
#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct DirEntry {
    /// SFN(Short file name) of the object
    pub name: [u8; 23],
    /// attribute of the object
    pub attr: DirEntryAttr,
    /// first cluster of the file
    pub first_cluster: u32,
    /// file size in bytes
    pub file_size: u32,
}

impl DirEntry {
    /// implement a new function for DirEntry, function's parameter is a slice of 32 bytes, then return a packed DirEntry struct
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

    /// increase_file_size: increase the file size
    pub fn increase_file_size(&mut self, size: u32) {
        self.file_size += size;
    }

    /// decrease_file_size: decrease the file size
    pub fn decrease_file_size(&mut self, size: u32) {
        self.file_size -= size;
    }

    /// is_valid: check if the directory entry is valid
    pub fn is_valid(&self) -> bool {
        self.name[0] != 0xE5 && self.name[0] != 0x00
    }

    /// is_dir: check if the directory entry is a directory
    pub fn is_dir(&self) -> bool {
        self.attr.contains(DirEntryAttr::Directory)
    }

    /// is_file: check if the directory entry is a file
    pub fn is_file(&self) -> bool {
        !self.is_dir()
    }

    /// name: get the name of the directory entry
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

    /// set_name: set the name of the directory entry
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

/// A Struct representing a FAT entry's type
#[derive(PartialEq)]
pub enum FatMarker {
    /// Free cluster
    Free,
    /// Reserved cluster
    Reserved,
    /// In use cluster
    InUse(u32),
    /// Bad cluster
    BadCluster,
    /// End of chain
    EndOfChain,
}

impl FatMarker {
    /// from_value: convert a u32 value to a FatMarker
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

/// convert a u8 array to a string
pub fn convert_to_u8_array(s: &str) -> Option<[u8; 23]> {
    let bytes = s.as_bytes();
    if bytes.len() > 23 {
        return None;
    }
    let mut array = [0u8; 23];
    array[..bytes.len()].copy_from_slice(bytes);
    Some(array)
}
