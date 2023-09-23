//! [ArceOS](https://github.com/rcore-os/arceos) filesystem module.
//!
//! It provides unified filesystem operations for various filesystems.
//!
//! # Cargo Features
//!
//! - `fatfs`: Use [FAT] as the main filesystem and mount it on `/`. This feature
//!    is **enabled** by default.
//! - `devfs`: Mount [`axfs_devfs::DeviceFileSystem`] on `/dev`. This feature is
//!    **enabled** by default.
//! - `ramfs`: Mount [`axfs_ramfs::RamFileSystem`] on `/tmp`. This feature is
//!    **enabled** by default.
//! - `myfs`: Allow users to define their custom filesystems to override the
//!    default. In this case, [`MyFileSystemIf`] is required to be implemented
//!    to create and initialize other filesystems. This feature is **disabled** by
//!    by default, but it will override other filesystem selection features if
//!    both are enabled.
//!
//! [FAT]: https://en.wikipedia.org/wiki/File_Allocation_Table
//! [`MyFileSystemIf`]: fops::MyFileSystemIf

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod mounts;
mod root;

pub mod api;
pub mod fops;

use axdiskfs::{disk, layout, sector};
use axdriver::{prelude::*, AxDeviceContainer};

/// Initializes filesystems by block devices.
pub fn init_filesystems(mut blk_devs: AxDeviceContainer<AxBlockDevice>) {
    info!("Initialize filesystems...");

    let dev = blk_devs.take_one().expect("No block device found!");
    info!("  use block device 0: {:?}", dev.device_name());
    #[cfg(any(feature = "myfs", feature = "fatfs"))]
    self::root::init_rootfs(self::dev::Disk::new(dev));
    #[cfg(feature = "diskfs")]
    self::root::init_my_rootfs(disk::Disk::new(dev));
}

/// Initializes sector manager by block devices.
pub fn init_sector_manager(disk: disk::Disk) -> Result<sector::SectorManager, DevError> {
    let sector = sector::SectorManager::new(disk);
    let boot_sector = layout::BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors_count: 32,
        total_sectors32: 4096,
        fat_count: 2,
        sectors_per_fat32: 31, //
        root_cluster: 2,
        root_dir_sectors_count: 1,
        fsinfo_sector: 1,
        reserved: [0; 488],
    };
    let fs_info_sector = layout::FSInfoSector::new(3967, 3);
    sector
        .write_sector_seq(&boot_sector.to_bytes())
        .expect("failed to write boot sector");
    sector
        .write_sector_seq(&fs_info_sector.to_bytes())
        .expect("failed to write fs info sector");
    Ok(sector)
}
