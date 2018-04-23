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
    file_ptr: usize,

    // FIXME: Fill me in.
}

impl File {
    pub fn new(name: String, vfat: Shared<VFat>, first_cluster: Cluster,
               metadata: Metadata) -> Self {
        File {
            name: name,
            vfat: vfat,
            first_cluster: first_cluster,
            metadata: metadata,
            file_ptr: 0
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
        self.metadata.size as u64
    }

}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//        println!("name {}, first clulster {:?} size {}, fileptr {}", self.name(), self.first_cluster, self.metadata.size, self.file_ptr);
        if self.metadata.size == 0 {
            return Ok(0);
        }

        let mut v = Vec::new();
        let _read = self.vfat.borrow_mut().read_chain(self.first_cluster, &mut v)?;
//        assert_eq!(read, self.metadata.size);

        let file_left = self.metadata.size as usize - self.file_ptr;
        let can_read = min(file_left, buf.len());
//        println!("can read {}", can_read);
        buf[..can_read].copy_from_slice(&v[self.file_ptr..self.file_ptr+can_read]);
        self.file_ptr += can_read;
//        for i in 0..can_read {
//            buf[i] = v.as_slice()[i];
//        }
        Ok(can_read)
    }

}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
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
        unimplemented!("File::seek()")
    }
}
