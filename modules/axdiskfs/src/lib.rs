//! RAM filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is based on [`axfs_vfs`].

#![cfg_attr(not(test), no_std)]

extern crate alloc;
use crate::sector::SectorManager;
use alloc::sync::Arc;
// use once_cell::sync::OnceCell;
// use lazy_static::lazy_static;
use lazy_init::LazyInit;

pub mod config;
pub mod dir;
pub mod disk;
pub mod diskfs;
pub mod file;
pub mod layout;
pub mod macros;
pub mod sector;

// pub static FS: OnceCell<Arc<diskfs::CCFileSystem>> = OnceCell::new();

pub static FS: LazyInit<Arc<diskfs::CCFileSystem>> = LazyInit::new();

pub fn initialize_fs(sector_manager: SectorManager) {
    let fs = diskfs::CCFileSystem::new(Some(sector_manager));
    fs.init().expect("failed to init filesystem");
    FS.init_by(Arc::new(fs));
}
