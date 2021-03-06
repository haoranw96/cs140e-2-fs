use std::{io, fmt};
use std::collections::HashMap;
use std::cmp::min;

use traits::BlockDevice;

#[derive(Debug, Default)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool
}

#[derive(Debug)]
pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedDevice
        where T: BlockDevice + 'static
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        if self.device.sector_size() == self.partition.sector_size {
            (virt, 1)
        } else if virt < self.partition.start {
            (virt, 1)
        } else {
            let factor = self.partition.sector_size / self.device.sector_size();
            let logical_offset = virt - self.partition.start;
            let physical_offset = logical_offset * factor;
            let physical_sector = self.partition.start + physical_offset;
            (physical_sector, factor)
        }
    }

    fn read_entry_from_dev(&mut self, sector: u64)
        -> io::Result<CacheEntry> {
        let (phy_sec, factor) = self.virtual_to_physical(sector);
        let mut data = Vec::with_capacity((self.device.sector_size() * factor) as usize);
//        println!("virt {} phys {} factor {}", sector, phy_sec, factor);
        for i in 0..factor {
//            println!("reading physical sector {}", phy_sec + i);
            self.device.read_all_sector(phy_sec + i, &mut data)?;
        }
        let entry = CacheEntry {
            data : data,
            dirty : false,
        };
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
        if !self.cache.contains_key(&sector) {
            let entry = self.read_entry_from_dev(sector)?;
            self.cache.insert(sector, entry);
        }
        Ok(&mut self.cache.get_mut(&sector).unwrap().data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
//        println!("getting sector {}", sector);
        if !self.cache.contains_key(&sector) {
            let entry = self.read_entry_from_dev(sector)?;
            self.cache.insert(sector, entry);
        }
        Ok(&self.cache.get(&sector).unwrap().data)
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedDevice {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let sec = self.get(n)?;
        let len = min(sec.len(), buf.len());
        buf[..len].copy_from_slice(&sec[..len]);
        Ok(len)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < self.sector_size() as usize {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                                      "buffer too small"));
        }
        let sec = self.get_mut(n)?;
        let len = min(sec.len(), buf.len());
        sec[..len].copy_from_slice(&buf[..len]);
        Ok(len)
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
//            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .field("partition", &self.partition)
            .finish()
    }
}
