use alloc::collections::BTreeMap;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloc::sync::{Arc, Weak};

use spin::RwLock;

use super::config::*;
use crate::file::{File, FileNode};
use crate::layout::{convert_to_u8_array, DirEntry, DirEntryAttr};
use crate::size_of_struct;
use crate::FS;
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use driver_block::DevError;

/// Basic Dir Struct for DirNode
#[derive(Clone)]
pub struct Dir {
    /// entries: a vector of DirEntry
    pub entries: Vec<DirEntry>,
    /// is_root: if dir is root, is_root = true, else is_root = false
    is_root: bool,
}

impl Dir {
    /// new_root: create a new Dir Struct for root
    pub fn new_root(entries: &Vec<DirEntry>) -> Self {
        Self {
            entries: entries.clone(),
            is_root: true,
        }
    }

    /// new: create a new Dir Struct
    pub fn new(first_cluster: u32, parent_first_cluster: u32) -> Self {
        let mut entries = Vec::new();
        entries.push(DirEntry {
            name: convert_to_u8_array(".").unwrap(),
            attr: DirEntryAttr::Directory,
            first_cluster,
            file_size: 0,
        });
        entries.push(DirEntry {
            name: convert_to_u8_array("..").unwrap(),
            attr: DirEntryAttr::Directory,
            first_cluster: parent_first_cluster,
            file_size: 0,
        });
        Self {
            entries,
            is_root: false,
        }
    }

    /// size: return the size of Dir
    fn size(&self) -> u32 {
        self.entries.iter().fold(0, |acc, entry| {
            acc + if entry.is_valid() { entry.file_size } else { 0 }
        })
    }

    /// get_entry_by_index: return the entry by index
    pub fn get_entry_by_index(&self, index: usize) -> Option<DirEntry> {
        Some(self.entries[index])
    }

    /// get_self_first_cluster: return the first cluster of self
    pub fn get_self_first_cluster(&self) -> u32 {
        self.get_entry_by_index(0).unwrap().first_cluster
    }

    /// find_next_free_entry: find the next free entry in entries
    pub fn find_next_free_entry(&self) -> Option<usize> {
        let start_index = if self.is_root { 0 } else { 2 };
        self.entries
            .iter()
            .enumerate()
            .skip(start_index)
            .find(|&(_, entry)| !entry.is_valid())
            .map(|(index, _)| index)
    }

    /// get_entry_by_name: return the entry by name
    pub fn get_entry_by_name(&self, name: &str) -> Option<usize> {
        self.entries.iter().enumerate().find_map(|(i, entry)| {
            if entry.name().as_deref() == Some(name) {
                Some(i)
            } else {
                None
            }
        })
    }

    /// set_entries: set entries
    pub fn set_entries(&mut self, entries: Vec<DirEntry>) {
        self.entries = entries;
    }

    /// update_file_size: update file_size of entry
    pub fn update_file_size(&mut self, file_name: &str, file_size: u32) -> Result<(), DevError> {
        let start_index = if self.is_root { 0 } else { 2 };
        if let Some(entry) = self
            .entries
            .iter_mut()
            .skip(start_index)
            .find(|e| e.name().unwrap_or_default() == file_name)
        {
            entry.file_size = file_size;
            Ok(())
        } else {
            Err(DevError::Unsupported)
        }
    }

    /// update_entries_from_disk: read all entries from disk and update entries
    pub fn update_entries_from_disk(&mut self) -> Result<(), DevError> {
        let mut new_entries = Vec::new();
        let mut curr_cluster = self.get_entry_by_index(0).unwrap().first_cluster;
        let dir_entry_size = size_of_struct!(DirEntry);
        let fs_mutex = FS.try_get().expect("FS not initialized");

        while fs_mutex.is_end(curr_cluster) {
            // if curr_cluster is Bad cluster
            if fs_mutex.is_bad_cluster(curr_cluster) {
                return Err(DevError::Unsupported);
            }
            let cluster_data = fs_mutex.read_cluster(curr_cluster)?;
            let mut cluster_entries = Vec::new();
            for i in 0..cluster_data.len() / dir_entry_size {
                let entry = DirEntry::new(&cluster_data[i * dir_entry_size..]);
                cluster_entries.push(entry);
            }
            new_entries.append(&mut cluster_entries);
            curr_cluster = fs_mutex.get_fat_entry(curr_cluster)?;
        }
        // todo: if previous cluster numbers can't hold all entries, allocate new cluster
        self.set_entries(new_entries);
        Ok(())
    }

    /// write_entries_to_disk: write all entries to disk
    pub fn write_entries_to_disk(&mut self) -> Result<(), DevError> {
        let mut curr_cluster = self.get_entry_by_index(0).unwrap().first_cluster;
        let dir_entry_size = size_of_struct!(DirEntry);
        let mut entries_idx = 0;
        let fs_arc = FS.try_get().expect("FS not initialized");

        while fs_arc.is_end(curr_cluster) {
            if fs_arc.is_bad_cluster(curr_cluster) {
                return Err(DevError::Unsupported);
            }

            let mut cluster_data = fs_arc.read_cluster(curr_cluster)?;

            for i in 0..cluster_data.len() / dir_entry_size {
                cluster_data[i * dir_entry_size..(i + 1) * dir_entry_size]
                    .copy_from_slice(self.entries[entries_idx].as_bytes());
            }

            entries_idx += 1;

            fs_arc.write_cluster(curr_cluster, &cluster_data[..])?;
            curr_cluster = fs_arc.get_fat_entry(curr_cluster)?;
        }

        Ok(())
    }

    // add_entry: will add a entry to entries, find a entry that name[0] == 0xE5 or name[0] == 0x00,if can't find create a new one after the last entry
    pub fn add_entry(&mut self, entry: DirEntry) -> Result<(), DevError> {
        let index = self.find_next_free_entry();
        match index {
            Some(index) => {
                self.entries[index] = entry;
            }
            None => {
                if self.entries.len() >= DIRECTORY_MAX_ENTRIES_NUM {
                    return Err(DevError::Unsupported);
                }
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    /// delete_entry: delete entry by name, set name[0] = 0xE5, free cluster
    pub fn delete_entry(&mut self, name: &str) -> Result<(), DevError> {
        let index = self.get_entry_by_name(name);
        let fs_arc = FS.try_get().expect("FS not initialized");
        match index {
            Some(index) => {
                self.entries[index].name[0] = 0xE5;
                // free cluster
                let mut curr_cluster = self.entries[index].first_cluster;
                while !fs_arc.is_end(curr_cluster) {
                    if fs_arc.is_bad_cluster(curr_cluster) {
                        return Err(DevError::Unsupported);
                    }
                    let next_cluster = fs_arc.get_fat_entry(curr_cluster)?;
                    fs_arc.free_cluster(curr_cluster)?;
                    curr_cluster = next_cluster;
                }
            }
            None => {
                return Err(DevError::Unsupported);
            }
        }
        Ok(())
    }

    /// is_entry_dir: return true if entry is dir, else return false
    pub fn is_entry_dir(&self, name: &str) -> Result<bool, DevError> {
        let index = self.get_entry_by_name(name);
        match index {
            Some(index) => Ok(self.entries[index].is_dir()),
            None => Err(DevError::Unsupported),
        }
    }

    /// update_entry: update entry by index
    pub fn update_entry(&mut self, index: u32, entry: DirEntry) -> Result<(), DevError> {
        if index <= 1 || index >= self.entries.len() as u32 {
            return Err(DevError::Unsupported);
        }
        self.entries[index as usize] = entry;
        Ok(())
    }

    /// update_entry_name: update entry's name by original_name, if can't find original_name, return Err
    pub fn update_entry_name(
        &mut self,
        original_name: &str,
        target_name: &str,
    ) -> Result<(), DevError> {
        if let Some(index) = self.entries.iter().position(|entry| {
            if let Some(entry_name) = entry.name() {
                entry_name == original_name
            } else {
                false
            }
        }) {
            let mut entry = self.entries[index];
            entry.name = convert_to_u8_array(target_name).unwrap();
            self.entries[index] = entry;
            Ok(())
        } else {
            Err(DevError::Unsupported)
        }
    }
}

/// DirNode: a struct that can represent a dir in VFS
pub struct DirNode {
    this: Weak<DirNode>,
    dir: RwLock<Dir>,
    name: RwLock<String>,
    parent: RwLock<Weak<DirNode>>,
    file_children: RwLock<BTreeMap<String, Arc<FileNode>>>,
    dir_children: RwLock<BTreeMap<String, Arc<DirNode>>>,
}

impl DirNode {
    /// new: create a new DirNode
    pub fn new(dir: Dir, name: String, parent: Option<Weak<DirNode>>) -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            dir: RwLock::new(dir),
            name: RwLock::new(name),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            file_children: RwLock::new(BTreeMap::new()),
            dir_children: RwLock::new(BTreeMap::new()),
        })
    }

    /// get_name: return the name of DirNode
    pub fn get_name(&self) -> String {
        self.name.read().clone()
    }

    /// get_total_size: return the total size of DirNode
    pub fn get_total_size(&self) -> u32 {
        self.dir.read().size()
    }

    /// update_file_size: update file_size of child_file_name
    pub fn update_child_file_size(
        &self,
        child_file_name: &str,
        file_size: u32,
    ) -> Result<(), DevError> {
        self.dir
            .write()
            .update_file_size(child_file_name, file_size)
    }

    /// is_empty: return true if DirNode is empty, else return false
    pub fn is_empty(&self) -> bool {
        self.file_children.read().is_empty() && self.dir_children.read().is_empty()
    }

    /// is_dir_child_empty: return true if DirNode's dir_children is empty, else return false
    fn is_dir_child_empty(&self, name: &str) -> Option<bool> {
        let children = self.dir_children.read();
        match children.get(name) {
            Some(child) => Some(child.is_empty()),
            None => None,
        }
    }

    /// self_rename: rename self
    pub fn self_rename(&self, target_name: &str) {
        *self.name.write() = target_name.to_string();
    }

    /// is_file_child_empty: return true if DirNode's file_children is empty, else return false
    fn is_file_child_empty(&self, name: &str) -> Option<bool> {
        let children = self.file_children.read();
        match children.get(name) {
            Some(child) => Some(child.is_empty()),
            None => None,
        }
    }

    /// is_child_empty: return true if DirNode's child is empty, else return false
    pub fn is_child_empty(&self, name: &str) -> Option<bool> {
        if let Some(is_empty) = self.is_dir_child_empty(name) {
            return Some(is_empty);
        }
        if let Some(is_empty) = self.is_file_child_empty(name) {
            return Some(is_empty);
        }
        None
    }

    // a function that can parse dir 's DirEntry ,then extract info to the children
    pub fn update_children(&self) -> Result<(), DevError> {
        let self_dir = self.dir.read();
        let self_entries_len = self_dir.entries.len();
        let is_root = self_dir.is_root;
        if self_entries_len > DIRECTORY_MAX_ENTRIES_NUM {
            return Err(DevError::Unsupported);
        }
        // if dir is root, start_index = 0, else start_index = 2
        let start_index = if is_root { 0 } else { 2 };

        for i in start_index..self_entries_len {
            let entry = self_dir.get_entry_by_index(i).unwrap();
            if entry.is_valid() {
                let name = entry.name().unwrap();
                match entry.is_dir() {
                    true => {
                        let child = DirNode::new(
                            Dir::new(entry.first_cluster, self_dir.get_self_first_cluster()),
                            name.to_string(),
                            Some(self.this.clone()),
                        );
                        self.add_dir_child(&name, child)?;
                    }
                    false => {
                        let child = FileNode::new(
                            File::new(entry.first_cluster),
                            name.to_string(),
                            Some(self.this.clone()),
                        );
                        self.add_file_child(&name, Arc::new(child))?;
                    }
                };
            }
        }
        Ok(())
    }

    /// add_dir_child: add a dir child to DirNode
    pub fn add_dir_child(&self, name: &str, child: Arc<DirNode>) -> Result<(), DevError> {
        self.dir_children.write().insert(name.to_string(), child);
        Ok(())
    }

    /// add_file_child: add a file child to DirNode
    pub fn add_file_child(&self, name: &str, child: Arc<FileNode>) -> Result<(), DevError> {
        self.file_children.write().insert(name.to_string(), child);
        Ok(())
    }

    /// create_dir_child: create a dir child to DirNode
    pub fn create_dir_child(&self, name: &str) -> Result<(), DevError> {
        let fs_arc = FS.try_get().expect("FS not initialized");
        let entry = DirEntry {
            name: convert_to_u8_array(name).ok_or(DevError::Unsupported)?,
            attr: DirEntryAttr::Directory,
            first_cluster: fs_arc
                .allocate_cluster_at_start()
                .ok_or(DevError::Unsupported)?,
            file_size: 0u32,
        };

        self.dir.write().add_entry(entry)?;
        let child = DirNode::new(
            Dir::new(
                entry.first_cluster,
                self.dir.read().get_self_first_cluster(),
            ),
            name.to_string(),
            Some(self.this.clone()),
        );
        self.add_dir_child(name, child)?;
        Ok(())
    }

    /// create_file_child: create a file child to DirNode
    pub fn create_file_child(&self, name: &str) -> Result<(), DevError> {
        let fs_arc = FS.try_get().expect("FS not initialized");
        let entry = DirEntry {
            name: convert_to_u8_array(name).ok_or(DevError::Unsupported)?,
            attr: DirEntryAttr::Archive,
            first_cluster: fs_arc
                .allocate_cluster_at_start()
                .ok_or(DevError::Unsupported)?,
            file_size: 0,
        };
        self.dir.write().add_entry(entry)?;
        let child = FileNode::new(
            File::new(entry.first_cluster),
            name.to_string(),
            Some(self.this.clone()),
        );
        self.add_file_child(name, Arc::new(child))?;
        Ok(())
    }

    /// create_child: create a child to DirNode
    pub fn remove_file_child(&self, name: &str) -> Result<(), DevError> {
        // find name's location in DirNode's entries, set name[0] = 0xE5, then update children
        self.dir.write().delete_entry(name)?;
        self.file_children.write().remove(name);
        Ok(())
    }

    /// remove_dir_child: remove a dir child to DirNode
    pub fn remove_dir_child(&self, name: &str) -> Result<(), DevError> {
        self.dir.write().delete_entry(name)?;
        self.dir_children.write().remove(name);
        Ok(())
    }

    /// remove_file_child: remove a file child to DirNode
    fn rename_file_child(&self, original_name: &str, target_name: &str) -> Result<(), DevError> {
        self.dir
            .write()
            .update_entry_name(original_name, target_name)?;
        let mut children = self.file_children.write();
        let original_child = children.remove(original_name).unwrap();
        original_child.self_rename(target_name);
        children.insert(target_name.to_string(), original_child);
        Ok(())
    }

    /// rename_dir_child: rename a dir child to DirNode
    fn rename_dir_child(&self, original_name: &str, target_name: &str) -> Result<(), DevError> {
        self.dir
            .write()
            .update_entry_name(original_name, target_name)?;
        let mut children = self.dir_children.write();
        let original_child = children.remove(original_name).unwrap();
        original_child.self_rename(target_name);
        children.insert(target_name.to_string(), original_child);
        Ok(())
    }

    /// rename_child: rename a child to DirNode
    pub fn rename_child(&self, original_name: &str, target_name: &str) -> Result<(), DevError> {
        if self.dir.read().is_entry_dir(original_name)? {
            self.rename_dir_child(original_name, target_name)
        } else {
            self.rename_file_child(original_name, target_name)
        }
    }

    /// inner_parent: return the parent of DirNode
    pub fn inner_parent(&self) -> Option<Arc<DirNode>> {
        self.parent.read().upgrade()
    }

    /// find_dir_child: find a dir child to DirNode
    pub fn find_dir_child(&self, name: &str) -> Result<Arc<DirNode>, DevError> {
        match self.dir_children.read().get(name) {
            Some(child) => Ok(child.clone()),
            None => Err(DevError::Unsupported),
        }
    }

    /// find_file_child: find a file child to DirNode
    pub fn find_file_child(&self, name: &str) -> Result<Arc<FileNode>, DevError> {
        match self.file_children.read().get(name) {
            Some(child) => Ok(child.clone()),
            None => Err(DevError::Unsupported),
        }
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            VfsNodePerm::from_bits_truncate(0o755),
            VfsNodeType::Dir,
            512,
            0,
        ))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.inner_parent().map(|parent| parent as VfsNodeRef)
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        let node = match name {
            "" | "." => Ok(self.clone() as VfsNodeRef),
            ".." => self.parent().ok_or(VfsError::NotFound),
            _ => {
                if let Ok(file) = self.find_file_child(name) {
                    Ok(file as VfsNodeRef)
                } else if let Ok(dir) = self.find_dir_child(name) {
                    Ok(dir as VfsNodeRef)
                } else {
                    Err(VfsError::NotFound)
                }
            }
        }?;

        if let Some(rest) = rest {
            node.lookup(rest)
        } else {
            Ok(node)
        }
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let dir_children = self.dir_children.read();
        let file_children = self.file_children.read();
        let is_root = self.dir.read().is_root;

        let entries = self.dir.read().entries.clone();

        if start_idx >= entries.len() {
            return Ok(0);
        }

        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let idx = start_idx + i;

            if is_root {
                if idx >= entries.len() {
                    return Ok(i);
                }

                let entry = &entries[idx];
                if let Some(entry_name) = entry.name() {
                    let (name, ty) = if dir_children.contains_key(&entry_name) {
                        (Some(entry_name.to_string()), VfsNodeType::Dir)
                    } else if file_children.contains_key(&entry_name) {
                        (Some(entry_name.to_string()), VfsNodeType::File)
                    } else {
                        continue;
                    };

                    if let Some(name) = name {
                        *out_entry = VfsDirEntry::new(&name, ty);
                    }
                } else {
                    continue;
                }
            } else {
                match idx {
                    0 => *out_entry = VfsDirEntry::new(".", VfsNodeType::Dir),
                    1 => *out_entry = VfsDirEntry::new("..", VfsNodeType::Dir),
                    _ => {
                        if idx - 2 < entries.len() {
                            let entry = &entries[idx - 2];
                            if let Some(entry_name) = entry.name() {
                                let (name, ty) = if dir_children.contains_key(&entry_name) {
                                    (Some(entry_name.to_string()), VfsNodeType::Dir)
                                } else if file_children.contains_key(&entry_name) {
                                    (Some(entry_name.to_string()), VfsNodeType::File)
                                } else {
                                    continue;
                                };

                                if let Some(name) = name {
                                    *out_entry = VfsDirEntry::new(&name, ty);
                                }
                            } else {
                                continue;
                            }
                        } else {
                            return Ok(i);
                        }
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    // use recursive method to create dir/file via create_file and create_dir
    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.create(rest, ty),
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(rest, ty),
                _ => {
                    // let subdir = self.find_dir_child(name).map_err(|e| match e {
                    //     _ => VfsError::Unsupported,
                    // })?;
                    let subdir = self.find_dir_child(name).or_else(|_| {
                        self.create_dir_child(name).map_err(|e| match e {
                            _ => VfsError::Unsupported,
                        })?;
                        self.find_dir_child(name).map_err(|e| match e {
                            _ => VfsError::Unsupported,
                        })
                    })?;
                    subdir.create(rest, ty)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Ok(())
        } else {
            match ty {
                VfsNodeType::File => self.create_file_child(name).map_err(|e| match e {
                    _ => VfsError::Unsupported,
                }),
                VfsNodeType::Dir => self.create_dir_child(name).map_err(|e| match e {
                    _ => VfsError::Unsupported,
                }),
                _ => Err(VfsError::Unsupported),
            }
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.remove(rest),
                ".." => self.parent().ok_or(VfsError::NotFound)?.remove(rest),
                _ => {
                    let subdir = self.find_dir_child(name).map_err(|e| match e {
                        _ => VfsError::NotFound,
                    })?;
                    subdir.remove(rest)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Err(VfsError::InvalidInput)
        } else {
            let is_dir = self.dir.read().is_entry_dir(name).map_err(|e| match e {
                _ => VfsError::NotFound,
            })?;
            if is_dir {
                if self.is_dir_child_empty(name).unwrap() {
                    self.remove_dir_child(name).map_err(|e| match e {
                        _ => VfsError::Unsupported,
                    })?;
                } else {
                    return Err(VfsError::DirectoryNotEmpty);
                }
            } else {
                self.remove_file_child(name).map_err(|e| match e {
                    _ => VfsError::Unsupported,
                })?;
            }
            Ok(())
        }
    }
    axfs_vfs::impl_vfs_dir_default! {}
}

/// split_path: split path to name and rest, for example: /a/b/c -> (a, Some(b/c))
fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}
