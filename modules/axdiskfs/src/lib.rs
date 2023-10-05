//! A simple fat32-like filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is inspired by [`..\crate\axfs_ramfs`].

#![cfg_attr(not(test), no_std)]

extern crate alloc;
use crate::sector::SectorManager;
use alloc::sync::Arc;
use lazy_init::LazyInit;

/// Platform-specific constants and parameters axdiskfs.
pub mod config;
/// Directory operations.
pub mod dir;
/// Disk operations.
pub mod disk;
/// Disk filesystem.
pub mod diskfs;
/// File operations.
pub mod file;
/// Filesystem layout.
pub mod layout;
/// Macros.
pub mod macros;
/// Sector operations.
pub mod sector;

/// A CCFileSystem global living example, used by File and Dir.
pub static FS: LazyInit<Arc<diskfs::CCFileSystem>> = LazyInit::new();

/// Initializes filesystems by Sector Manager.
pub fn initialize_fs(sector_manager: SectorManager) {
    let fs = diskfs::CCFileSystem::new(Some(sector_manager));
    fs.init().expect("failed to init filesystem");
    FS.init_by(Arc::new(fs));
}
