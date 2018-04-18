use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;
use std::mem;
use std::collections::HashMap;
use std::string::String;

use traits;
use util::VecExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    name: String,
    first_cluster: Cluster,
    vfat: Shared<VFat>,
    // FIXME: Fill me in.
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attr: Attributes,
    win_nt_reserved: u8,
    ctime_tenth_sec: u8,
    ctime: Timestamp,
    adate: Date,
    cluster_num_hi: u16,
    mtime: Timestamp,
    cluster_num_lo: u16,
    file_sz: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    id: u8,
    reserved1: [u8; 9],
    attr: Attributes,
    reserved2: [u8; 21],
    // FIXME: Fill me in.
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
        for entry in self.entries()? {
            if entry.name().eq_ignore_ascii_case(name_str) {
                return Ok(entry);
            }
        }
        return Err(io::Error::new(io::ErrorKind::NotFound, "name not found"));
    }
}

pub struct VFatDirEntryIter {
    entries: Vec<VFatUnknownDirEntry>,
    vfat: Shared<VFat>,
    iter: usize,
}

impl Iterator for VFatDirEntryIter {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        let first_entry = self.iter;
        let mut name_hash = HashMap::new();
        
        for (i, entry) in self.entries.as_slice()[self.iter..].iter().enumerate() {
            if entry.id == 0x00 { return None; }
            else if entry.id == 0xE5 { continue } 


            self.iter += 1;

            if entry.attr.lfn() {
                let mut v = Vec::new();
                let entry = unsafe{ &*(entry as *const VFatUnknownDirEntry as *const VFatLfnDirEntry) };
                v.extend_from_slice(&entry.chars1);
                v.extend_from_slice(&entry.chars2);
                v.extend_from_slice(&entry.chars3);
                name_hash.insert(i, v);
            } else {
                break;
            }
        }

        let entry = unsafe{ &*(&self.entries.as_slice()[self.iter] 
                               as *const VFatUnknownDirEntry 
                               as *const VFatRegularDirEntry) };

        let name = if name_hash.len() == 0 {
            let mut name_vec = Vec::new();
            name_vec.extend_from_slice(&entry.name);
            if entry.ext != [b'\0', b'\0', b'\0'] {
                name_vec.push(b'.');
                name_vec.extend_from_slice(&entry.ext);
            }
            String::from_utf8(name_vec).ok()?
        } else {
            let mut name_vec = Vec::new();
            for seq in 0..name_hash.len() {
                let mut chars = name_hash.remove(&seq)?;
                name_vec.append(&mut chars);
            }
            String::from_utf16(&name_vec).ok()?
        };
        let first_cluster = (entry.cluster_num_hi as u32) << 16 | entry.cluster_num_lo as u32;

        if entry.attr.directory() {
            Some(Entry::Dir(Dir{
                name: name,
                first_cluster: Cluster::from(first_cluster),
                vfat: self.vfat.clone(),
            }))
        } else {
            Some(Entry::File(File{
                name: name,
                first_cluster: Cluster::from(first_cluster),
                vfat: self.vfat.clone(),
            }))
        }
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
        let mut buf = Vec::new();
        self.vfat
            .borrow_mut()
            .read_chain(self.first_cluster, &mut buf)
            .and_then(|read| {
                Ok({
                    let len = read * mem::size_of::<u8>() / mem::size_of::<VFatUnknownDirEntry>();
                    let v = unsafe {Vec::from_raw_parts(buf.as_mut_ptr() as *mut _,
                                                        len, len)};
                    
                    VFatDirEntryIter{entries: v, vfat: self.vfat.clone() ,iter: 0}
                })
            })
    }
}
