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

    // FIXME: Fill me in.
}

impl File {
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
        unimplemented!()
    }

}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut v = Vec::new();
        let read = self.vfat.borrow_mut().read_chain(self.first_cluster, &mut v)?;
        let can_read = min(buf.len(), read);
        for i in 0..can_read {
            buf[i] = v.as_slice()[i];
        }
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
