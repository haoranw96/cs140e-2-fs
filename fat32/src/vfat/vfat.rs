use std::io;
use std::slice;
use std::path::{Path, Component};
use std::cmp::min;
use std::mem;

use util::SliceExt;
use mbr::{MasterBootRecord, PartitionEntry, CHS};
use vfat::{Shared, Cluster, File, Dir, Entry, FatEntry, Error, Status};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use traits::{FileSystem, BlockDevice};

#[derive(Debug)]
pub struct VFat {
    pub device: CachedDevice,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub sectors_per_fat: u32,
    pub fat_start_sector: u64,
    pub data_start_sector: u64,
    pub root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
        where T: BlockDevice + 'static
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        let bpb_start = mbr.first_fat32().ok_or(Error::NotFound)?
                           .relative_sector as u64;
        let ebpb = BiosParameterBlock::from(&mut device, bpb_start)?;
//        println!("{:?}", mbr);
//        println!("{:?}", ebpb);
        let fat_start_sector = bpb_start + ebpb.num_reserved_sectors as u64;
        let data_start_sector = fat_start_sector +
            (ebpb.num_fat as u64) * ebpb.sectors_per_fat() as u64;
        let dev = CachedDevice::new(device, 
                                    Partition{
                                        start: bpb_start,
                                        sector_size: ebpb.bytes_per_sector as u64,
                                    });

        Ok(Shared::new(VFat {
            device: dev,
            bytes_per_sector: ebpb.bytes_per_sector,
            sectors_per_cluster: ebpb.sectors_per_cluster,
            sectors_per_fat: ebpb.sectors_per_fat(),
            fat_start_sector: bpb_start + ebpb.num_reserved_sectors as u64,
            data_start_sector: data_start_sector,
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
//        println!("vfat {:?}", self);
//        println!("cluster {}, self.bytes_per_sector {}, self.device.sector_size {}", cluster.get_index(), self.bytes_per_sector, self.device.sector_size());
        let cluster_start = (cluster.get_index() - 2) as u64 * self.sectors_per_cluster as u64 + self.data_start_sector;
        let start_sector = cluster_start + offset as u64;
        let end_sector = cluster_start + self.sectors_per_cluster as u64;
        let can_read = buf.len() as u64 / self.bytes_per_sector as u64;
        let can_read_end = min(end_sector, start_sector + can_read);

        let mut read = 0;
        for i in start_sector..can_read_end {
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
//        println!("read chain from {:?}", start);
        loop {
            let buflen = buf.len();
            buf.resize(buflen + self.bytes_per_sector as usize * self.sectors_per_cluster as usize, 0);
            read += self.read_cluster(cur_cluster, 0, &mut buf[read..])?;
            match self.fat_entry(cur_cluster)?.status() {
                Status::Data(next_cluster) => {
                    cur_cluster = next_cluster; }
                Status::Eoc(_) => {
                    return Ok(read);
                },
                _ => return Err(io::Error::new(io::ErrorKind::Other, "sector unreadable"))
            }
        }
    }

    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let entries_per_sector = self.bytes_per_sector as usize / mem::size_of::<FatEntry>();
        let nth_sec_in_fat = cluster.get_index() as usize / entries_per_sector;
        let index_in_sector = cluster.get_index() as usize % entries_per_sector;
        let sec = self.device.get(nth_sec_in_fat as u64 + self.fat_start_sector as u64)?;
        let entries: &[FatEntry] = unsafe { sec.cast() };
//        println!("cluster: {:?} entries_per_sector {} nth_sec_in_fat {} entries.len {}, index_in_sector {}, entries {:?}",
//                 cluster, entries_per_sector, nth_sec_in_fat, entries.len(), index_in_sector, entries);
//        println!("{:?}", entries);

        let entry = entries[index_in_sector];
        Ok(&entries[index_in_sector])
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = File;
    type Dir = Dir;
    type Entry = Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        use vfat::Entry as vfatEntry;
        use traits::Entry;

        let mut cur_dir = vfatEntry::Dir(Dir::root(self.clone()));

        for comp in path.as_ref().components() {
//            println!("comp: {:?}", comp);
            match comp {
                Component::RootDir => { },
                Component::Normal(name) => {
                    cur_dir = cur_dir.as_dir()
                                     .ok_or(io::Error::new(io::ErrorKind::NotFound, 
                                                           "File not found"))?
                                     .find(name)?
                }
                Component::CurDir => unimplemented!("CurDir"),
                Component::ParentDir => unimplemented!("ParentDir"),
                Component::Prefix(_) => unimplemented!("Prefix"),
            }
        }
        Ok(cur_dir)
//        unimplemented!("FileSystem::open()")
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
