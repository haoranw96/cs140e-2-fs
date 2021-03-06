use std::ffi::OsStr;
//use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
//use std::borrow::Cow;
use std::io;
use std::string::String;
use std::str;
use std::vec::IntoIter;

use traits;
use util::VecExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub first_cluster: Cluster,
    pub vfat: Shared<VFat>,
    pub metadata: Metadata,
    // FIXME: Fill me in.
}

impl Dir {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn root(vfat: Shared<VFat>) -> Dir {
        Dir{
            name: String::from("/"),
            first_cluster: vfat.borrow().root_dir_cluster,
            vfat: vfat.clone(),
            metadata: Metadata::default(),
        }
    }

}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: Attributes,
    win_nt_reserved: u8,
    ctime_tenth_sec: u8,
    ctime: Time,
    cdate: Date,
    adate: Date,
    cluster_num_hi: u16,
    mtime: Time,
    mdate: Date,
    cluster_num_lo: u16,
    file_sz: u32,
}

impl VFatRegularDirEntry {
    pub fn metadata(&self) -> Metadata {
        Metadata {
            attr: self.attr,
            ctime: Timestamp{
                time: self.ctime,
                date: self.cdate,
            },
            atime: Timestamp{
                time: Time(0),
                date: self.adate,
            },
            mtime: Timestamp{
                time: self.mtime,
                date: self.mdate,
            },
        }
    }

}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatLfnDirEntry {
    seq: u8,
    chars1: [u16; 5],
    attr: Attributes,
    lfn_type: u8,
    checksum: u8,
    chars2: [u16; 6],
    zero: u16,
    chars3: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VFatUnknownDirEntry {
    seq: u8,
    reserved1: [u8; 10],
    attr: Attributes,
    reserved2: [u8; 20],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use traits::Dir;
        use traits::Entry;

        let name_str = name.as_ref()
                           .to_str()
                           .ok_or(io::Error::new(io::ErrorKind::InvalidInput,
                                       "input contains invalid UTF-8 char")
                                  )?;
        self.entries()?
            .find(|entry| entry.name().eq_ignore_ascii_case(name_str))
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "name not found"))
    }
}

pub struct VFatDirEntryIter {
    entries: IntoIter<VFatDirEntry>,
    vfat: Shared<VFat>,
}

impl Iterator for VFatDirEntryIter {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        let mut lfn_vec = [0u16; 13 * 31]; // Max lfn length = 13 u16 * 31 entries
        let mut has_lfn = false;

        for ref entry in self.entries.by_ref() {
            let unknown_entry = unsafe { entry.unknown };
            if unknown_entry.seq == 0x00 {
                return None; 
            } else if unknown_entry.seq == 0xE5 {
                continue
            }

            if unknown_entry.attr.lfn() {
                let entry = unsafe { entry.long_filename };
                has_lfn = true;
                let seq = (entry.seq & 0x1F) as usize - 1;
                lfn_vec[seq * 13      ..seq * 13 + 5 ].copy_from_slice(&entry.chars1);
                lfn_vec[seq * 13 + 5  ..seq * 13 + 11].copy_from_slice(&entry.chars2);
                lfn_vec[seq * 13 + 11 ..seq * 13 + 13].copy_from_slice(&entry.chars3);
            } else {
                let entry = unsafe { entry.regular };
                let name = if !has_lfn {
                    let mut name = entry.name.clone();
                    let name = str::from_utf8(&name).ok()?.trim_right();
                    let ext = str::from_utf8(&entry.ext).ok()?.trim_right();

                    let mut name_str = String::from(name);
                    if ext.len() > 0 {
                        name_str.push_str(&".");
                        name_str.push_str(&ext);
                    }
//                    println!("shortname {}", &name_str);
                    name_str
                } else {
                    let len = lfn_vec.iter().position(|&c| c == 0x0000 || c == 0xFFFF)
                                     .unwrap_or_else(||lfn_vec.len());
                    String::from_utf16(&lfn_vec[..len]).ok()?
                };

                let first_cluster = Cluster::from((entry.cluster_num_hi as u32) << 16 
                                                 | entry.cluster_num_lo as u32);

//                println!("name {}", &name);
                return Some(if entry.attr.directory() {
                    Entry::Dir(Dir{
                        name: name,
                        first_cluster: first_cluster,
                        vfat: self.vfat.clone(),
                        metadata: entry.metadata(),
                    })
                } else {
                    Entry::File(File::new(name, self.vfat.clone(), first_cluster, entry.metadata(), entry.file_sz))
                });
            }
        }
        None
    }
}

// FIXME: Implement `trait::Dir` for `Dir`.
impl traits::Dir for Dir {
    /// The type of entry stored in this directory.
    type Entry = Entry;

    /// An type that is an iterator over the entries in this directory.
    type Iter = VFatDirEntryIter;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
//        println!("{:?}", self.vfat.clone());
//        println!("entries per sector: {}", self.vfat.borrow().bytes_per_sector / mem::size_of::<VFatUnknownDirEntry>() as u16);
        let mut buf = Vec::new();
        self.vfat.borrow_mut()
            .read_chain(self.first_cluster, &mut buf)
            .and_then(|_read|
                Ok(VFatDirEntryIter{entries: unsafe { buf.cast() }.into_iter(),
                                    vfat: self.vfat.clone()})
            )
    }
}
