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

impl Attributes {
    const READ_ONLY: u8 = 0x01;
    const HIDDEN   : u8 = 0x02;
    const SYSTEM   : u8 = 0x04;
    const VOLUME_ID: u8 = 0x08;
    const DIRECTORY: u8 = 0x10;
    const ARCHIVE  : u8 = 0x20;
    const LFN      : u8 = Self::READ_ONLY | Self::HIDDEN | Self::SYSTEM | Self::VOLUME_ID;

    pub fn read_only(&self) -> bool {
        self.0 & Self::READ_ONLY == Self::READ_ONLY
    }

    pub fn hidden(&self) -> bool {
        self.0 & Self::HIDDEN == Self::HIDDEN
    }

    pub fn system(&self) -> bool {
        self.0 & Self::SYSTEM == Self::SYSTEM
    }

    pub fn volume_id(&self) -> bool {
        self.0 & Self::VOLUME_ID == Self::VOLUME_ID
    }

    pub fn directory(&self) -> bool {
        self.0 & Self::DIRECTORY == Self::DIRECTORY
    }

    pub fn archive(&self) -> bool {
        self.0 & Self::ARCHIVE == Self::ARCHIVE
    }

    pub fn lfn(&self) -> bool {
        self.0 & Self::LFN == Self::LFN
    }


}

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
    pub attr: Attributes,
    pub ctime_tenth_sec: u8,
    pub ctime: Timestamp,
    pub adate: Date,
    pub mtime: Timestamp,
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
    fn read_only(&self) -> bool { self.attr.0 & 0x01 == 0x01 }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool { self.attr.0 & 0x02 == 0x02 }

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
