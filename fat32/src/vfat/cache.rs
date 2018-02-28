use std::{io, fmt};
use std::collections::HashMap;
use std::cmp::min;

use traits::BlockDevice;

#[derive(Debug, Default)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    pub fn new<T: BlockDevice + 'static>(device: T) -> CachedDevice {
        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new()
        }
    }

    fn insert_entry<T: BlockDevice + 'static + ?Sized>(device: &mut Box<T>, sector: u64)
        -> io::Result<CacheEntry> {
        let mut entry = CacheEntry::default();
        device.read_sector(sector, entry.data.as_mut_slice())?;
        Ok(entry)
    }
    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        Ok(self.cache
               .entry(sector)
               .and_modify(|e| e.dirty = true)
               .or_insert(Self::insert_entry(&mut self.device, sector)?)
               .data
               .as_mut_slice())
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        Ok(self.cache
               .entry(sector)
               .or_insert(Self::insert_entry(&mut self.device, sector)?)
               .data
               .as_slice())
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`.
impl BlockDevice for CachedDevice {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let sec = self.get(n)?;
        let len = min(sec.len(), buf.len());
        buf.copy_from_slice(&sec[..len]);
        Ok(len)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < self.sector_size() as usize {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                      "buffer too small"));
        }
        let sec = self.get_mut(n)?;
        let len = min(sec.len(), buf.len());
        sec.copy_from_slice(&buf[..len]);
        Ok(len)
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
