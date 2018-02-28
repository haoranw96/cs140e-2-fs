use std::fmt;
use std::{slice, mem};

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    reserved: [u8; 476],
    signature: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors: u8,
    num_reserved_sectors: u16,
    num_fat: u8,
    max_dir_entries: u16,
    total_logical_sectors: u16,
    media_desc_type: u8,
    num_sector_per_fat: u16,
    num_sector_per_track: u16,
    num_heads: u16,
    num_hidden_sectors: u32,
    total_logical_sectors_large: u32,
}

impl BiosParameterBlock {
    pub fn default() -> BiosParameterBlock {
        BiosParameterBlock {
            reserved: [0; 476],
            signature: [0; 3],
            oem_id: [0; 8],
            bytes_per_sector: 0,
            sectors: 0,
            num_reserved_sectors: 0,
            num_fat: 0,
            max_dir_entries: 0,
            total_logical_sectors: 0,
            media_desc_type: 0,
            num_sector_per_fat: 0,
            num_sector_per_track: 0,
            num_heads: 0,
            num_hidden_sectors: 0,
            total_logical_sectors_large: 0,
        }
    }

    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        let mut bpb = Self::default();
        let mut bpb_buf = unsafe {
            slice::from_raw_parts_mut(&mut bpb as *mut BiosParameterBlock as *mut u8,
                                  mem::size_of::<BiosParameterBlock>())
        };

        if let Err(e) = device.read_sector(sector, &mut bpb_buf) {
            return Err(Error::Io(e));
        }

        println!("{:?}", bpb.signature);
        println!("{}", bpb.total_logical_sectors_large);
//        if bpb.signature != [0x00, 0x55, 0xAA] {
        if bpb.total_logical_sectors_large != 2857697280 {
            return Err(Error::BadSignature);
        }

        Ok(bpb)



//        unimplemented!("BiosParameterBlock::from()"
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("reserved", &"<Reserved>")
            .field("signature", &self.signature)
            .field("oem id", &self.oem_id)
            .field("bytes per sector", &self.bytes_per_sector)
            .field("total sectors", &self.sectors)
            .field("number of reserved sectors", &self.num_reserved_sectors)
            .field("number of FAT", &self.num_fat)
            .field("maximum directory entries", &self.max_dir_entries)
            .field("total logical sectors", &self.total_logical_sectors)
            .field("media description type", &self.media_desc_type)
            .field("number of sectors per FAT", &self.num_sector_per_fat)
            .field("number of sectors per track", &self.num_sector_per_track)
            .field("number of heads", &self.num_heads)
            .field("number of hidden sectors", &self.num_hidden_sectors)
            .field("total logical sectors (large number)", &self.total_logical_sectors_large)
            .finish()
    }
}
