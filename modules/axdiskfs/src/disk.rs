use axdriver::{prelude::*, AxBlockDevice};

/// The block size of the disk. Default to 512 bytes.
const BLOCK_SIZE: usize = 512;

/// Seek from where.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Seek from the start of the disk.
    Start(u64),
    /// Seek from the current position.
    End(i64),
    /// Seek from the end of the disk.
    Current(i64),
}

/// A disk device with a cursor.
pub struct Disk {
    /// The current block id.
    block_id: u64,
    /// The current offset within the block.
    offset: usize,
    /// The underlying block device.
    dev: AxBlockDevice,
}

impl Disk {
    /// Create a new disk.
    pub fn new(dev: AxBlockDevice) -> Self {
        assert_eq!(BLOCK_SIZE, dev.block_size());
        Self {
            block_id: 0,
            offset: 0,
            dev,
        }
    }

    /// Get the block size of the disk.
    pub fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    /// Get the size of the disk.
    pub fn size(&self) -> u64 {
        self.dev.num_blocks() * BLOCK_SIZE as u64
    }

    /// Get the position of the cursor.
    pub fn position(&self) -> u64 {
        self.block_id * BLOCK_SIZE as u64 + self.offset as u64
    }

    /// Set the position of the cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.block_id = pos / BLOCK_SIZE as u64;
        self.offset = pos as usize % BLOCK_SIZE;
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(&mut self, buf: &mut [u8]) -> DevResult<usize> {
        let read_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev
                .read_block(self.block_id, &mut buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            self.dev.read_block(self.block_id, &mut data)?;
            buf[..count].copy_from_slice(&data[start..start + count]);

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, buf: &[u8]) -> DevResult<usize> {
        let write_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev.write_block(self.block_id, &buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            self.dev.read_block(self.block_id, &mut data)?;
            data[start..start + count].copy_from_slice(&buf[..count]);
            self.dev.write_block(self.block_id, &data)?;

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(write_size)
    }

    /// Read from the disk, returns the number of bytes read.
    pub fn read(&mut self, mut buf: &mut [u8]) -> DevResult<usize> {
        let mut read_len = 0;
        while !buf.is_empty() {
            match self.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_) => return Err(DevError::Unsupported),
            }
        }
        Ok(read_len)
    }

    /// Write to the disk, returns the number of bytes written.
    pub fn write(&mut self, mut buf: &[u8]) -> DevResult<usize> {
        let mut write_len = 0;
        while !buf.is_empty() {
            match self.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_) => return Err(DevError::Unsupported),
            }
        }
        Ok(write_len)
    }

    /// Seek the cursor, returns the new position.
    pub fn seek(&mut self, pos: SeekFrom) -> DevResult<u64> {
        let size = self.size();
        let new_pos = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.position().checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(DevError::Unsupported)?;
        if new_pos > size {
            return Err(DevError::Unsupported);
        }
        self.set_position(new_pos);
        Ok(new_pos)
    }

    /// Read at a global offset, returns the number of bytes read.
    pub fn read_at(&mut self, global_offset: u64, buf: &mut [u8]) -> DevResult<usize> {
        let pos = self.position();
        self.seek(SeekFrom::Start(global_offset))?;
        let read_len = self.read(buf)?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(read_len)
    }

    /// Write at a global offset, returns the number of bytes written.
    pub fn write_at(&mut self, global_offset: u64, buf: &[u8]) -> DevResult<usize> {
        let pos = self.position();
        self.seek(SeekFrom::Start(global_offset))?;
        let write_len = self.write(buf)?;
        self.seek(SeekFrom::Start(pos))?;
        Ok(write_len)
    }
}
