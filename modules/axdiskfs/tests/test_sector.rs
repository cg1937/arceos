use axdiskfs::{disk, sector};
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
fn test_sector() {
    println!("test sector from axdiskfs...");

    let disk = make_disk().expect("failed to load disk image");
    let disk = disk::Disk::new(disk);
    let sector = sector::SectorManager::new(disk);
    println!("sector size = {} bytes", sector.sector_size());
    println!("sector count = {}", sector.sector_count());

    // test read_sector_at()
    println!("test read_sector_at()");
    let mut buf = [0u8; 512];
    let read_size = sector
        .read_sector_at(0, &mut buf)
        .expect("failed to read sector");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 512);
    println!("pos = {}", sector.position());

    // prepare for 8 + 16 + 32
    // let buf = [1, 2, 3, 4, 5, 6, 7];
    let mut data: Vec<u8> = Vec::new();
    let num1: u8 = 1;
    let num2: u16 = 2;
    let num3: u32 = 3;

    data.push(num1);
    data.extend(&num2.to_le_bytes());
    data.extend(&num3.to_le_bytes());
    let data_slice: &[u8] = &data;

    let write_size = sector
        .write_sector_at(0, data_slice)
        .expect("failed to write sector");
    assert_eq!(write_size, 7);
    println!("pos = {}", sector.position());

    // test read_8()
    println!("test read_8()");
    let read_8_buf = sector.read_8(0).expect("failed to read 8 bytes");
    println!("read_8_buf = {:?}", read_8_buf);
    println!("pos = {}", sector.position());

    // test read_16()
    println!("test read_16()");
    let read_16_buf = sector.read_16(1).expect("failed to read 16 bytes");
    println!("read_16_buf = {:?}", read_16_buf);
    println!("pos = {}", sector.position());

    // test read_32()
    println!("test read_32()");
    let read_32_buf = sector.read_32(3).expect("failed to read 32 bytes");
    println!("read_32_buf = {:?}", read_32_buf);
    println!("pos = {}", sector.position());

    // test write_8(), write_16(), write_32()
    println!("test write_8(), write_16(), write_32()");
    let num1: u8 = 4;
    let num2: u16 = 5;
    let num3: u32 = 6;
    sector.write_8(0, num1).expect("failed to write 8 bytes");
    sector.write_16(1, num2).expect("failed to write 16 bytes");
    sector.write_32(3, num3).expect("failed to write 32 bytes");
    println!("pos = {}", sector.position());
    let mut buf = [0u8; 7];
    let read_size = sector
        .read_sector_at(0, &mut buf)
        .expect("failed to read sector");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 7);
    println!("pos = {}", sector.position());
    let read_8_buf = sector.read_8(0).expect("failed to read 8 bytes");
    println!("read_8_buf = {:?}", read_8_buf);
    let read_16_buf = sector.read_16(1).expect("failed to read 16 bytes");
    println!("read_16_buf = {:?}", read_16_buf);
    let read_32_buf = sector.read_32(3).expect("failed to read 32 bytes");
    println!("read_32_buf = {:?}", read_32_buf);

    // test write_8_seq(), write_16_seq(), write_32_seq()
    println!("test write_8_seq(), write_16_seq(), write_32_seq()");
    let num1: u8 = 7;
    let num2: u16 = 8;
    let num3: u32 = 9;
    sector
        .write_8_seq(num1)
        .expect("failed to write 8 bytes sequentially");
    sector
        .write_16_seq(num2)
        .expect("failed to write 16 bytes sequentially");
    sector
        .write_32_seq(num3)
        .expect("failed to write 32 bytes sequentially");
    println!("pos = {}", sector.position());
    let mut buf = [0u8; 7];
    let read_size = sector
        .read_sector_at(0, &mut buf)
        .expect("failed to read sector");
    println!("buf = {:?}", buf);
    assert_eq!(read_size, 7);
    println!("reset pos = {}", sector.position());
    sector.set_position(0);
    println!("pos = {}", sector.position());
    let read_8_buf = sector.read_8_seq().expect("failed to read 8 bytes");
    println!("read_8_buf = {:?}", read_8_buf);
    let read_16_buf = sector.read_16_seq().expect("failed to read 16 bytes");
    println!("read_16_buf = {:?}", read_16_buf);
    let read_32_buf = sector.read_32_seq().expect("failed to read 32 bytes");
    println!("read_32_buf = {:?}", read_32_buf);

    // test write_sector_seq() and read_sector_seq()
    println!("test write_sector_seq() and read_sector_seq()");
    sector.set_position(0);
    let mut data: Vec<u8> = Vec::new();
    let num1: u8 = 10;
    let num2: u16 = 11;
    let num3: u32 = 12;

    data.push(num1);
    data.extend(&num2.to_le_bytes());
    data.extend(&num3.to_le_bytes());
    let data_slice: &[u8] = &data;
    sector
        .write_sector_seq(data_slice)
        .expect("failed to write sector sequentially");
    println!("pos = {}", sector.position());
    sector.set_position(0);
    println!("reset pos = {}", sector.position());
    let read_sector_buf = sector.read_sector_seq().expect("failed to read sector");
    println!("read_buf = {:?}", &read_sector_buf[0..7]);
    println!("pos = {}", sector.position());
}
