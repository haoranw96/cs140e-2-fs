use std::{fmt, io, mem, slice};

use traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct CHS {
    starting_head: u8,
    starting_sector: u8,
    starting_cylinder: u8,
}

impl CHS {
    pub fn default() -> CHS {
        CHS {
            starting_head : 0,
            starting_sector : 0,
            starting_cylinder : 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct PartitionEntry {
    boot_indicator: u8,
    start_chs: CHS,
    partition_type: u8,
    end_chs: CHS,
    relative_sector: u32,
    total_sectors: u32,
}

impl PartitionEntry {
    pub fn default() -> PartitionEntry {
        PartitionEntry {
            boot_indicator : 0,
            start_chs: CHS::default(),
            partition_type : 0,
            end_chs: CHS::default(),
            relative_sector : 0,
            total_sectors : 0,
        }
    }
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    partition_table: [PartitionEntry; 4],
    signature: [u8; 2],
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
    pub fn default() -> MasterBootRecord {
        MasterBootRecord{
            bootstrap: [0; 436],
            disk_id: [0; 10],
            partition_table: [PartitionEntry::default(); 4],
            signature: [0; 2],
        }
    }

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
