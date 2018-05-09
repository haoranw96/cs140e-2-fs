use std::cmp::{min};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub vfat: Shared<VFat>,
    pub first_cluster: Cluster,
    pub metadata: Metadata,
    pub size: u32,
    file_ptr: u32,

    // FIXME: Fill me in.
}

impl File {
    pub fn new(name: String, vfat: Shared<VFat>, first_cluster: Cluster,
               metadata: Metadata, file_sz: u32) -> Self {
        File {
            name: name,
            vfat: vfat,
            first_cluster: first_cluster,
            metadata: metadata,
            file_ptr: 0,
            size: file_sz
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl traits::File for File {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!()
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size as u64
    }

}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.size == 0 {
            return Ok(0);
        }

        let mut v = Vec::new();
        let _read = self.vfat.borrow_mut().read_chain(self.first_cluster, &mut v)?;

        let file_left = self.size - self.file_ptr;
        let can_read = min(file_left, buf.len() as u32);
        buf[..can_read as usize]
            .copy_from_slice(&v[self.file_ptr as usize..(self.file_ptr+can_read) as usize]);
        self.file_ptr += can_read;
        Ok(can_read as usize)
    }

}

impl io::Write for File {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!() 
    }
}

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        use traits::File;
        match pos {
            SeekFrom::Start(offset) => {
                if offset > self.size() {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                        format!("invalid position {}", offset)))
                } else {
                    self.file_ptr = offset as u32;
                    Ok(self.file_ptr as u64)
                }
            }
            SeekFrom::End(offset) => {
                let new_ptr = if offset.is_negative() {
                    self.size().checked_sub((-offset) as u64)
                } else {
                    self.size().checked_add(offset as u64)
                
                }.ok_or(io::Error::new(io::ErrorKind::InvalidInput,
                        format!("invalid position {}", offset)))? as u32;
                        
                if new_ptr > self.size() as u32 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                        format!("invalid position {}", offset)))
                } else {
                    self.file_ptr = new_ptr as u32;
                    Ok(self.file_ptr as u64)
                }
            },
            SeekFrom::Current(offset) => {
                let new_ptr = if offset.is_negative() {
                    self.file_ptr.checked_sub((-offset) as u32)
                } else {
                    self.file_ptr.checked_add(offset as u32)
                }.ok_or(io::Error::new(io::ErrorKind::InvalidInput,
                        format!("invalid position {}", offset)))?;
                if new_ptr > self.size() as u32 {
                    Err(io::Error::new(io::ErrorKind::InvalidInput,
                        format!("invalid position {}", offset)))
                } else {
                    self.file_ptr = new_ptr as u32;
                    Ok(self.file_ptr as u64)
                }
            }
        
        }
    }
}
