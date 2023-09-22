use axdiskfs::{disk, diskfs, initialize_fs, layout, sector, FS};
use driver_block::ramdisk::RamDisk;
const IMG_PATH: &str = "resources/myimage.img";
// format CCFileSystem on myimage.img
fn init_sector_manager() -> std::io::Result<sector::SectorManager> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    let disk = disk::Disk::new(RamDisk::from(&data));
    let sector = sector::SectorManager::new(disk);
    println!(
        "sector size = {} bytes",
        core::mem::size_of::<layout::BootSector>()
    );
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
    println!("end init sector manager");
    Ok(sector)
}

#[test]
fn test_sector() {
    println!("test sector from axdiskfs...");
    let sector = init_sector_manager().expect("failed to init sector manager");
    initialize_fs(sector);
    let fs_arc = FS.try_get().expect("FS not initialized");
    let res = fs_arc.is_end(0x0fffffff);
    assert_eq!(res, true);
    let res = fs_arc.is_end(0x0ffffff8);
    assert_eq!(res, true);
    let res = fs_arc.is_end(0x0ffffff7);
    assert_eq!(res, false);
    let res = fs_arc.is_bad_cluster(0x0ffffff7);
    assert_eq!(res, true);

    let root = fs_arc.root_dir_node().unwrap();

    let res = root.is_empty();
    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    assert_eq!(res, true);

    println!("root name = {:?}", root.get_name());

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    let res = root.create_file_child("test.txt");
    let test_file = root.find_file_child("./test.txt").unwrap();

    println!("test file name = {:?}", test_file.get_name());
    println!("test file is empty = {:?}", test_file.is_empty());
    println!("---------root is empty = {:?}", root.is_empty());

    let buf = [1u8; 7];
    let res = test_file.write_at_test(0, &buf).unwrap();
    assert_eq!(res, 7);

    println!("test file is empty = {:?}", test_file.is_empty());
    println!("test file size is {:?}", test_file.get_size());

    let mut buf = [0u8; 7];
    let res = test_file.read_at_test(0, &mut buf).unwrap();
    // assert_eq!(res, 7);
    println!("res = {:?}", res);
    println!("buf = {:?}", buf);

    let res = fs_arc.get_fat_entry(3).unwrap();
    println!("res = {:x}", res);

    let res = fs_arc.get_fat_entry(2).unwrap();
    println!("res = {:x}", res);

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    println!("---------------------------------");
    let buf = [2u8; 521];
    let res = test_file.write_at_test(5, &buf).unwrap();
    assert_eq!(res, 521);

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    let all_read_buf = test_file.read_all_test().unwrap();
    println!("all_read_buf = {:?}", all_read_buf);

    println!("**********************************");
    let size = test_file.get_size();
    println!("size = {:?}", size);

    let res = test_file.truncate_test(10);
    assert!(res.is_ok());
    let size = test_file.get_size();
    println!("size = {:?}", size);

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    let all_read_buf = test_file.read_all_test().unwrap();
    println!("all_read_buf = {:?}", all_read_buf);

    println!("&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&");

    root.create_file_child("test2.txt");
    let test2_file = root.find_file_child("./test2.txt").unwrap();
    println!("test2 file name = {:?}", test2_file.get_name());

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    let buf = [3u8; 521];
    let res = test2_file.write_at_test(0, &buf).unwrap();
    assert_eq!(res, 521);

    println!("next free cluster = {:?}", fs_arc.get_next_free_cluster());

    let all_read_buf = test2_file.read_all_test().unwrap();
    println!("all_read_buf = {:?}", all_read_buf);

    let root_2 = test2_file.parent().unwrap();

    println!("root_2 name = {:?}", root_2.get_name());
    println!("root_2 sub total size = {:?}", root_2.get_total_size());

    println!("root_2 is empty = {:?}", root_2.is_empty());

    let res = root_2.create_dir_child("test_dir1");
    assert!(res.is_ok());

    println!(
        "after create sub dir next free cluster = {:?}",
        fs_arc.get_next_free_cluster()
    );

    let test_dir1 = root_2.find_dir_child("./test_dir1").unwrap();

    println!("test_dir1 name = {:?}", test_dir1.get_name());

    let res = test_dir1.create_file_child("test3.txt");

    println!(
        "after create sub file next free cluster = {:?}",
        fs_arc.get_next_free_cluster()
    );

    // let test_file3 = test_dir1.find_file_child("./test3.txt").unwrap();
    // println!("test_file3 name = {:?}", test_file3.get_name());

    let test_file3 = root_2.find_file_child("./test_dir1/test3.txt").unwrap();
    println!("test_file3 name = {:?}", test_file3.get_name());
    println!("test file3 size = {:?}", test_file3.get_size());

    let buf = [4u8; 10];
    test_file3.write_at_test(0, &buf).unwrap();
    println!("test file3 size = {:?}", test_file3.get_size());

    println!("test dir 1 size = {:?}", test_dir1.get_total_size());

    println!("root 2 total size = {:?}", root_2.get_total_size());

    println!("root 2 is empty = {:?}", root_2.is_empty());

    println!(
        "after create sub file next free cluster = {:?}",
        fs_arc.get_next_free_cluster()
    );

    println!(
        "test dir 1's test file3 is empty = {:?}",
        test_dir1.is_child_empty("test3.txt").unwrap()
    );

    println!(
        "root's test dir1  is empty = {:?}",
        root.is_child_empty("test_dir1").unwrap(),
    );

    println!("===========test delete===========");

    let res = root.remove_file_child("test.txt");
    assert!(res.is_ok());

    let res = root_2.find_file_child("./test.txt");

    assert!(res.is_err());

    println!(
        "after delete file next free cluster = {:?}",
        fs_arc.get_next_free_cluster()
    );

    // test rename

    let res = root_2.rename_child("test_dir1", "test_dir1_1");
    assert!(res.is_ok());

    let res = root_2.find_dir_child("./test_dir1");
    assert!(res.is_err());

    let res = root_2.find_dir_child("./test_dir1_1");
    assert!(res.is_ok());

    let new_test_dir = root_2.find_dir_child("./test_dir1_1").unwrap();
    println!("new_test_dir name = {:?}", new_test_dir.get_name());

    let test3_file = root_2.find_file_child("./test_dir1_1/test3.txt").unwrap();
    println!("test3_file name = {:?}", test3_file.get_name());
    println!("test3_file size = {:?}", test3_file.get_size());
}
