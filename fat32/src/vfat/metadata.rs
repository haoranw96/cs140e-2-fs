use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

impl Date {
    pub fn year(&self) -> usize { (self.0 >> 9) as usize + 1980 }

    pub fn month(&self) -> u8 { ((self.0 & 0x1E0) >> 5) as u8 }

    pub fn day(&self) -> u8 { self.0 as u8 & 0x1F }
}

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(pub u16);

impl Time {
    pub fn hour(&self) -> u8 { (self.0 >> 11) as u8 }

    pub fn minute(&self) -> u8 { ((self.0 & 0x7E0) >> 5) as u8 }

    pub fn second(&self) -> u8 { (self.0 as u8 & 0x1F) * 2 }
}

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

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
        self.0 == Self::LFN
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
//    pub ctime_tenth_sec: u8,
    pub ctime: Timestamp,
    pub atime: Timestamp,
    pub mtime: Timestamp,
    pub size: u32,
}

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize { self.date.year() }

    fn month(&self) -> u8 { self.date.month() }

    fn day(&self) -> u8 { self.date.day() }

    fn hour(&self) -> u8 { self.time.hour() }

    fn minute(&self) -> u8 { self.time.minute() }

    fn second(&self) -> u8 { self.time.second()  }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    /// Whether the associated entry is read only.
    fn read_only(&self) -> bool { self.attr.read_only() }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool { self.attr.hidden() }

    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp { self.ctime }

    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp { self.atime }

    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp { self.mtime }

}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Metadata")
         .field("attr", &format!("{:?}", &self.attr))
//         .field("ctime_tenth_sec", &self.ctime_tenth_sec)
         .field("ctime", &self.ctime)
         .field("atime", &self.atime)
         .field("mtime", &self.mtime)
         .finish()
    }

}
