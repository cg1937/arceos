#![cfg(feature = "diskfs")]

mod test_common;

use axdiskfs::FS;
use axdriver::AxDeviceContainer;
use driver_block::ramdisk::RamDisk;

const IMG_PATH: &str = "resources/myimage.img";

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
}

#[test]
fn test_diskfs() {
    println!("Testing diskfs with ramdisk ...");

    let disk = make_disk().expect("failed to load disk image");
    axtask::init_scheduler(); // call this to use `axsync::Mutex`.
    axfs::init_filesystems(AxDeviceContainer::from_one(disk));

    let fs_arc = FS.try_get().expect("FS not initialized");

    let root = fs_arc.root_dir_node().unwrap();
    root.create_file_child("short.txt").unwrap();
    let mut file = root.find_file_child("short.txt").unwrap();
    file.write_at_test(0, "Rust is cool\n".as_bytes());

    println!("string len: {}", "Rust is cool\n".len());

    root.create_file_child("long.txt").unwrap();
    let mut long_file = root.find_file_child("long.txt").unwrap();
    let mut big_string = String::new();
    for _ in 0..100 {
        big_string.push_str("Rust is cool\n");
    }

    long_file.write_at_test(0, big_string.as_bytes());

    root.create_dir_child("very-long-dir-name");

    let mut very_long_dir = root.find_dir_child("very-long-dir-name").unwrap();
    very_long_dir
        .create_file_child("very-long-file-name.txt")
        .unwrap();
    let mut very_long_file = very_long_dir
        .find_file_child("very-long-file-name.txt")
        .unwrap();

    very_long_file.write_at_test(0, "Rust is cool\n".as_bytes());

    root.create_dir_child("very");
    let mut very_dir = root.find_dir_child("very").unwrap();

    very_dir.create_dir_child("long").unwrap();
    let mut very_long_dir = very_dir.find_dir_child("long").unwrap();

    very_long_dir.create_dir_child("path").unwrap();
    let mut very_long_path = very_long_dir.find_dir_child("path").unwrap();

    very_long_path.create_file_child("test.txt").unwrap();
    let mut test_file = very_long_path.find_file_child("test.txt").unwrap();

    test_file.write_at_test(0, "Rust is cool\n".as_bytes());

    test_common::test_all();
}
