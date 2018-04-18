use std::io;
use std::path::Path;
use std::mem::size_of;
use std::cmp::min;
use std::mem;

use util::SliceExt;
use mbr::{MasterBootRecord, PartitionEntry, CHS};
use vfat::{Shared, Cluster, File, Dir, Entry, FatEntry, Error, Status};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use traits::{FileSystem, BlockDevice};

#[derive(Debug)]
pub struct VFat {
    device: CachedDevice,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
        where T: BlockDevice + 'static
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        let bpb_start = mbr.partition_table[0].start_chs.get_sector();
        let ebpb = BiosParameterBlock::from(&mut device, bpb_start as u64)?;
        let mut dev = CachedDevice::new(device, 
                                        Partition{
                                            start: mbr.partition_table[0].relative_sector as u64,
                                            sector_size: ebpb.bytes_per_sector as u64,
                                        });
        let first_fat32 = mbr.first_fat32().ok_or(Error::NotFound)?;
        let first_fat32_sec = first_fat32.relative_sector;

        Ok(Shared::new(VFat {
            device: dev,
            bytes_per_sector: ebpb.bytes_per_sector,
            sectors_per_cluster: ebpb.sectors_per_cluster,
            sectors_per_fat: {
                if ebpb.sectors_per_fat == 0 {
                    ebpb.sectors_per_fat_32
                } else {
                    ebpb.sectors_per_fat as u32
                }
            },
            fat_start_sector: first_fat32_sec as u64 + ebpb.num_reserved_sectors as u64,
            data_start_sector: ebpb.sectors_per_cluster as u64,
            root_dir_cluster: Cluster::from(ebpb.root_cluster)
        }))
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    //    fn read_cluster(
    //        &mut self,
    //        cluster: Cluster,
    //        offset: usize,
    //        buf: &mut [u8]
    //    ) -> io::Result<usize>;
    pub fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8])
        -> io::Result<usize> {
        let cluster_start = cluster.get_index() as u64 * self.sectors_per_cluster as u64 + self.fat_start_sector;
        let start_sector = cluster_start + offset as u64 / self.bytes_per_sector as u64;
        let end_sector = cluster_start + self.sectors_per_cluster as u64;
        let can_read = buf.len() / self.bytes_per_sector as usize;
        let mut read = 0;
        for i in start_sector..min(end_sector, start_sector + can_read as u64) {
            read += self.device.read_sector(i, &mut buf[read..])?;
        }
        Ok(read)
    }

    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    //    fn read_chain(
    //        &mut self,
    //        start: Cluster,
    //        buf: &mut Vec<u8>
    //    ) -> io::Result<usize>;
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut cur_cluster = start;
        let mut read = 0;
        loop {
            buf.reserve((self.bytes_per_sector as usize) * self.sectors_per_cluster as usize);
            read += self.read_cluster(cur_cluster, 0, &mut buf.as_mut_slice()[read..])?;
            match self.fat_entry(cur_cluster)?.status() {
                Status::Data(next_cluster) => {
                    cur_cluster = next_cluster;
                }
                Status::Eoc(_) => {
                    return Ok(read);
                },
                _ => return Err(io::Error::new(io::ErrorKind::Other, "sector unreadable"))
            }
        }
        unreachable!();
    }




    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let entries_per_sector = self.bytes_per_sector as usize * mem::size_of::<FatEntry>();
        let nth_sec_in_fat = cluster.get_index() as usize / entries_per_sector;
        let index_in_sector = cluster.get_index() as usize % entries_per_sector;
        let sec = self.device.get(nth_sec_in_fat as u64+ self.fat_start_sector as u64)?;
        let fat_entries : &[FatEntry] = unsafe{mem::transmute(sec)};
        Ok(&fat_entries[index_in_sector])
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = ::traits::Dummy;
    type Dir = ::traits::Dummy;
    type Entry = ::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
        where P: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
        where P: AsRef<Path>, Q: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
