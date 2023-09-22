//! fat32 filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is inspired by [`axfs_ramfs`].

#![cfg_attr(not(test), no_std)]

extern crate alloc;
use crate::sector::SectorManager;
use alloc::sync::Arc;
// use once_cell::sync::OnceCell;
// use lazy_static::lazy_static;
use lazy_init::LazyInit;

/// config mod: platform-specific constants and parameters for [axdiskfs].
pub mod config;
/// dir mod: directory operations.
pub mod dir;
/// disk mod: disk operations.
pub mod disk;
/// diskfs mod: disk filesystem.
pub mod diskfs;
/// file mod: file operations.
pub mod file;
/// layout mod: filesystem layout.
pub mod layout;
/// macros mod: macros.
pub mod macros;
/// sector mod: sector operations.
pub mod sector;

/// Alias of [`axdiskfs::diskfs::CCFileSystem`] living example.
pub static FS: LazyInit<Arc<diskfs::CCFileSystem>> = LazyInit::new();

/// Initializes filesystems by Sector Manager.
pub fn initialize_fs(sector_manager: SectorManager) {
    let fs = diskfs::CCFileSystem::new(Some(sector_manager));
    fs.init().expect("failed to init filesystem");
    FS.init_by(Arc::new(fs));
}
