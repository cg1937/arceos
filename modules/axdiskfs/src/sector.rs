use crate::disk::Disk;
use alloc::vec;
use alloc::vec::Vec;
use axdriver::prelude::*;
use spin::Mutex;

/// A sector manager warpper for disk.
pub struct SectorManager {
    inner: Mutex<Disk>,
}

impl SectorManager {
    /// Create a new sector manager.
    pub fn new(disk: Disk) -> Self {
        Self {
            inner: Mutex::new(disk),
        }
    }

    /// sector_size: return the size of sector
    pub fn sector_size(&self) -> usize {
        self.inner.lock().block_size()
    }

    /// sector_count: return the count of sector
    pub fn sector_count(&self) -> u64 {
        self.inner.lock().size()
    }

    /// position: return the position of sector
    pub fn position(&self) -> u64 {
        self.inner.lock().position()
    }

    /// set_position: set the position of sector
    pub fn set_position(&self, global_offset: u64) {
        self.inner.lock().set_position(global_offset);
    }

    /// read_sector_at: read a sector at global_offset, return the number of bytes read.
    pub fn read_sector_at(&self, global_offset: u64, buf: &mut [u8]) -> DevResult<usize> {
        self.inner.lock().read_at(global_offset, buf)
    }

    /// read_8: read a 8 byte at global_offset, return the data read.
    pub fn read_8(&self, global_offset: u64) -> DevResult<u8> {
        let mut buf = [0; 1];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(buf[0])
    }

    /// read_16: read a 16 byte at global_offset, return the data read.
    pub fn read_16(&self, global_offset: u64) -> DevResult<u16> {
        let mut buf = [0; 2];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// read_32: read a 32 byte at global_offset, return the data read.
    pub fn read_32(&self, global_offset: u64) -> DevResult<u32> {
        let mut buf = [0; 4];
        self.read_sector_at(global_offset, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// a read 8 byte in sequence, not use global_offset,return the data read.
    pub fn read_8_seq(&self) -> DevResult<u8> {
        let mut buf = [0; 1];
        self.inner.lock().read(&mut buf)?;
        Ok(buf[0])
    }

    /// read_16_seq: read a 16 byte in sequence, not use global_offset, return the data read.
    pub fn read_16_seq(&self) -> DevResult<u16> {
        let mut buf = [0; 2];
        self.inner.lock().read(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// read_32_seq: read a 32 byte in sequence, not use global_offset, return the data read.
    pub fn read_32_seq(&self) -> DevResult<u32> {
        let mut buf = [0; 4];
        self.inner.lock().read(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// read_sector_seq: read a sector in sequence, not use global_offset, return the data read.
    pub fn read_sector_seq(&self) -> DevResult<Vec<u8>> {
        let mut buf = vec![0; self.sector_size()];
        self.inner.lock().read(&mut buf)?;
        Ok(buf)
    }

    /// write_sector_at: write a sector at global_offset, return the number of bytes written.
    pub fn write_sector_at(&self, global_offset: u64, buf: &[u8]) -> DevResult<usize> {
        self.inner.lock().write_at(global_offset, buf)
    }

    /// write_8: write a 8 byte at global_offset, return the number of bytes written.
    pub fn write_8(&self, global_offset: u64, data: u8) -> DevResult {
        let buf = [data];
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    /// write_16: write a 16 byte at global_offset, return the number of bytes written.
    pub fn write_16(&self, global_offset: u64, data: u16) -> DevResult {
        let buf = data.to_le_bytes();
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    /// write_32: write a 32 byte at global_offset, return the number of bytes written.
    pub fn write_32(&self, global_offset: u64, data: u32) -> DevResult {
        let buf = data.to_le_bytes();
        self.write_sector_at(global_offset, &buf)?;
        Ok(())
    }

    /// write_8_seq: write a 8 byte in sequence, not use global_offset, return the number of bytes written.
    pub fn write_8_seq(&self, data: u8) -> DevResult {
        let buf = [data];
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    /// write_16_seq: write a 16 byte in sequence, not use global_offset, return the number of bytes written.
    pub fn write_16_seq(&self, data: u16) -> DevResult {
        let buf = data.to_le_bytes();
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    /// write_32_seq: write a 32 byte in sequence, not use global_offset, return the number of bytes written.
    pub fn write_32_seq(&self, data: u32) -> DevResult {
        let buf = data.to_le_bytes();
        self.inner.lock().write(&buf)?;
        Ok(())
    }

    /// write_sector_seq: write a sector in sequence, not use global_offset, return the number of bytes written.
    pub fn write_sector_seq(&self, buf: &[u8]) -> DevResult {
        self.inner.lock().write(buf)?;
        Ok(())
    }
}
