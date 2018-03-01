use std::{fmt, io, mem, slice};

use traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CHS {
    head: u8,
    sector: u8,
    cylinder: u8,
}

impl CHS {
    pub fn get_sector(&self) -> u8 {
        self.sector & 0b111111
    }
}


#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Default)]
pub struct PartitionEntry {
    pub boot_indicator: u8,
    pub start_chs: CHS,
    pub partition_type: u8,
    pub end_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
#[derive(Default)]
pub struct MasterBootRecord {
    bootstrap_1: [u64; 32],
    bootstrap_2: [u64; 22],
    bootstrap_3: [u32; 1],
    pub disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    pub signature: [u8; 2],
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut mbr = Self::default();
        let mut mbr_buf = unsafe {
            slice::from_raw_parts_mut(&mut mbr as *mut MasterBootRecord as *mut u8,
                                      mem::size_of::<MasterBootRecord>())
        };
        if let Err(e) = device.read_sector(0, &mut mbr_buf) {
            return Err(Error::Io(e));
        }

        if mbr.signature != [0x55, 0xAA] {
            return Err(Error::BadSignature);
        }

        for i in 0..mbr.partition_table.len() {
            let part_tab = mbr.partition_table[i];
            if part_tab.boot_indicator != 0 && part_tab.boot_indicator !=  0x80 {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(mbr)
    }

    pub fn first_fat32(&self) -> Option<&PartitionEntry> {
        for i in 0..self.partition_table.len() {
            let p = self.partition_table[i];
            if p.partition_type == 0xB || p.partition_type == 0xC {
                return Some(&self.partition_table[i])
            }
        }
        None
    }

}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("bootstrap", &"<bootstrap binary>")
            .field("disk_id", &self.disk_id)
            .field("partition_table", &self.partition_table)
            .field("signature", &self.signature)
            .finish()
    }
}
