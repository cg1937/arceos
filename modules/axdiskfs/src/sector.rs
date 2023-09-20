use crate::disk::Disk;
use alloc::vec;
use alloc::vec::Vec;
use axdriver::prelude::*;
use spin::Mutex;

pub struct SectorManager {
    inner: Mutex<Disk>,
}

impl SectorManager {
    pub fn new(disk: Disk) -> Self {
        Self {
            inner: Mutex::new(disk),
        }
    }

    pub fn sector_size(&self) -> usize {
        self.inner.lock().block_size()
    }

    pub fn sector_count(&self) -> u64 {
        self.inner.lock().size()
    }

    pub fn position(&self) -> u64 {
        self.inner.lock().position()
    }

    pub fn set_position(&self, global_offset: u64) {
        self.inner.lock().set_position(global_offset);
    }

    pub fn read_sector_at(&self, global_offset: u64, buf: &mut [u8]) -> DevResult<usize> {
        self.inner.lock().read_at(global_offset, buf)
    }

    pub fn read_8(&self, global_offset: u64) -> DevResult<u8> {
        let mut buf = [0; 1];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(buf[0])
    }

    pub fn read_16(&self, global_offset: u64) -> DevResult<u16> {
        let mut buf = [0; 2];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_32(&self, global_offset: u64) -> DevResult<u32> {
        let mut buf = [0; 4];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    // a read 8 byte in sequence, not use global_offset
    pub fn read_8_seq(&self) -> DevResult<u8> {
        let mut buf = [0; 1];
        self.inner.lock().read(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_16_seq(&self) -> DevResult<u16> {
        let mut buf = [0; 2];
        self.inner.lock().read(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_32_seq(&self) -> DevResult<u32> {
        let mut buf = [0; 4];
        self.inner.lock().read(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_sector_seq(&self) -> DevResult<Vec<u8>> {
        let mut buf = vec![0; self.sector_size()];
        self.inner.lock().read(&mut buf)?;
        Ok(buf)
    }

    pub fn write_sector_at(&self, global_offset: u64, buf: &[u8]) -> DevResult<usize> {
        self.inner.lock().write_at(global_offset, buf)
    }

    pub fn write_8(&self, global_offset: u64, data: u8) -> DevResult {
        let buf = [data];
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    pub fn write_16(&self, global_offset: u64, data: u16) -> DevResult {
        let buf = data.to_le_bytes();
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    pub fn write_32(&self, global_offset: u64, data: u32) -> DevResult {
        let buf = data.to_le_bytes();
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    pub fn write_8_seq(&self, data: u8) -> DevResult {
        let buf = [data];
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    pub fn write_16_seq(&self, data: u16) -> DevResult {
        let buf = data.to_le_bytes();
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    pub fn write_32_seq(&self, data: u32) -> DevResult {
        let buf = data.to_le_bytes();
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    pub fn write_sector_seq(&self, buf: &[u8]) -> DevResult {
        self.inner.lock().write(buf)?;
        Ok(())
    }
}
