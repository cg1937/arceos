use axdiskfs::disk;
use driver_block::ramdisk::RamDisk;

use std::process::Command;

const IMG_PATH: &str = "resources/myimage.img";

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
}

#[test]
fn test_disk() {
    let mut cmd = Command::new("sh");

    cmd.arg("./resources/test_make_diskfs_img.sh")
        .output()
        .expect("failed to execute process");
    println!("test disk from axdiskfs...");

    let disk = make_disk().expect("failed to load disk image");
    let mut disk = disk::Disk::new(disk);
    println!("size = {} bytes", disk.size());

    // test read on whole block
    println!("test read on whole block");
    let mut buf = [0u8; 512];
    let read_size = disk.read_one(&mut buf).expect("failed to read");
    assert_eq!(read_size, 512);
    println!("pos = {}", disk.position());

    // test read on partial block
    println!("test read on partial block");
    let mut buf = [0u8; 256];
    let read_size = disk.read_one(&mut buf).expect("failed to read");
    assert_eq!(read_size, 256);
    println!("pos = {}", disk.position());

    // reset position
    println!("reset position");
    disk.set_position(0);
    println!("pos = {}", disk.position());

    // test write on whole block
    println!("test write on whole block");
    let buf = [1u8; 512];
    let write_size = disk.write_one(&buf).expect("failed to write");
    assert_eq!(write_size, 512);
    println!("pos = {}", disk.position());
    disk.set_position(0);
    println!("pos = {}", disk.position());
    // test read on whole block
    let mut buf = [0u8; 512];
    let read_size = disk.read_one(&mut buf).expect("failed to read");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 512);
    println!("pos = {}", disk.position());

    // test write on partial block
    println!("test write on partial block");
    let buf = [2u8; 256];
    let write_size = disk.write_one(&buf).expect("failed to write");
    assert_eq!(write_size, 256);
    println!("pos = {}", disk.position());
    disk.set_position(512);
    println!("pos = {}", disk.position());
    // test read on partial block
    let mut buf = [0u8; 256];
    let read_size = disk.read_one(&mut buf).expect("failed to read");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 256);
    println!("pos = {}", disk.position());

    // test read 512+256 bytes use read()
    println!("test read 512+256 bytes use read()");
    disk.set_position(0);
    let mut buf = [0u8; 512 + 256];
    let read_size = disk.read(&mut buf).expect("failed to read");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 512 + 256);
    println!("pos = {}", disk.position());

    // test seek
    println!("test seek");
    disk.set_position(0);
    disk.seek(disk::SeekFrom::Start(512))
        .expect("failed to seek");
    println!("pos = {}", disk.position());
    disk.seek(disk::SeekFrom::Current(256))
        .expect("failed to seek");
    println!("pos = {}", disk.position());
    disk.seek(disk::SeekFrom::End(-256))
        .expect("failed to seek");
    println!("pos = {}", disk.position());
    assert_eq!(disk.position(), disk.size() - 256);
    disk.seek(disk::SeekFrom::End(0)).expect("failed to seek");
    println!("pos = {}", disk.position());
    assert_eq!(disk.position(), disk.size());
    let res = disk.seek(disk::SeekFrom::End(256));
    assert!(res.is_err());
    disk.seek(disk::SeekFrom::Start(0)).expect("failed to seek");
    println!("pos = {}", disk.position());
    disk.seek(disk::SeekFrom::Current(0))
        .expect("failed to seek");
    println!("pos = {}", disk.position());
    let res = disk.seek(disk::SeekFrom::Current(-256));
    assert!(res.is_err());
    println!("pos = {}", disk.position());
    disk.seek(disk::SeekFrom::Current(512))
        .expect("failed to seek");
    println!("pos = {}", disk.position());
    disk.seek(disk::SeekFrom::Current(-512))
        .expect("failed to seek");
    println!("pos = {}", disk.position());

    // test write_at() and read_at()
    println!("test write_at() and read_at()");
    disk.set_position(0);
    let buf = [3u8; 256];
    let write_size = disk.write_at(512, &buf).expect("failed to write_at()");
    assert_eq!(write_size, 256);
    println!("pos = {}", disk.position());
    disk.set_position(512);
    println!("pos = {}", disk.position());
    let mut buf = [0u8; 256];
    let read_size = disk.read_at(512, &mut buf).expect("failed to read_at()");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 256);
    println!("pos = {}", disk.position());

    // test read_at() on 1 byte
    println!("test read_at() on 1 byte");
    let mut buf = [0u8; 1];
    let read_size = disk.read_at(520, &mut buf).expect("failed to read_at()");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 1);
    println!("pos = {}", disk.position());

    // test write_at() on 1 byte
    println!("test write_at() on 1 byte");
    let buf = [4u8; 1];
    let write_size = disk.write_at(520, &buf).expect("failed to write_at()");
    assert_eq!(write_size, 1);
    println!("pos = {}", disk.position());
    let mut buf = [0u8; 1];
    let read_size = disk.read_at(520, &mut buf).expect("failed to read_at()");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 1);
    println!("pos = {}", disk.position());
}
