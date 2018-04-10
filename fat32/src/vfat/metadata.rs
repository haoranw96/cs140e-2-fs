use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[repr(C, packed)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub time: Time,
    pub date: Date,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    file_name: [u8; 8],
    file_ext: [u8; 3],
    file_attr: Attributes,
    win_nt_reserved: u8,
    ctime_tenth_sec: u8,
    ctime: Timestamp,
    adate: Date,
    cluster_num_hi: u16,
    mtime: Timestamp,
    cluster_num_lo: u16,
    file_sz: u32,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize { (self.date.0 >> 9) as usize + 1980 }

    fn month(&self) -> u8 { (self.date.0 as u8 & 0xF0) >> 5 }

    fn day(&self) -> u8 { self.date.0 as u8 & 0xF }

    fn hour(&self) -> u8 { (self.time.0 >> 11) as u8 }

    fn minute(&self) -> u8 { (self.time.0 >> 5) as u8 & 0x3F }

    fn second(&self) -> u8 { self.time.0 as u8 & 0xF }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    /// Whether the associated entry is read only.
    fn read_only(&self) -> bool { self.file_attr.0 & 0x01 == 0x01 }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool { self.file_attr.0 & 0x02 == 0x02 }

    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp { self.ctime }

    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp { 
        Timestamp {date: self.adate, time: Time(0)}
    }

    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp { self.mtime }

}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
