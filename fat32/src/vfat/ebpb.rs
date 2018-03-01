use std::fmt;
use std::{slice, mem};

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
#[derive(Default)]
pub struct BiosParameterBlock {
    /* BPB */
    jump_short_nop: [u8; 3],
    pub oem_id: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub num_reserved_sectors: u16,
    pub num_fat: u8,
    pub max_dir_entries: u16,
    pub total_logical_sectors: u16,
    pub media_desc_type: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub num_hidden_sectors: u32,
    pub total_logical_sectors_32: u32,
    /* EBPB */
    pub sectors_per_fat_32: u32,
    pub flags: u16,
    pub fat_version: u16,
    pub root_cluster: u32,
    pub fsinfo_sector: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_num: u8,
    pub win_nt_flag: u8,
    pub signature: u8,
    pub volumn_id: u32,
    pub volumn_label: [u8; 11],
    pub sys_id_str: [u8; 8],
    /* boot code separate into 3 parts to
     * make derive(Default) available*/
    boot_code_1: [u64; 32],
    boot_code_2: [u64; 20],
    boot_code_3: [u32; 1],
    bootable_signature: u16, 
}

impl BiosParameterBlock {
    fn modify_byte_order(bpb: BiosParameterBlock) -> BiosParameterBlock {
        // Too lazy to implement, should have no problem on pi
        bpb
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

        if bpb.bootable_signature != 0xAA55 {
            return Err(Error::BadSignature);
        }

        Ok(bpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
            .field("reserved", &"<Reserved>")
            .field("signature", &self.signature)
            .field("oem id", &self.oem_id)
            .field("bytes per sector", &self.bytes_per_sector)
            .field("total sectors", &self.sectors_per_cluster)
            .field("number of reserved sectors", &self.num_reserved_sectors)
            .field("number of FAT", &self.num_fat)
            .field("maximum directory entries", &self.max_dir_entries)
            .field("total logical sectors", &self.total_logical_sectors)
            .field("media description type", &self.media_desc_type)
            .field("number of sectors per FAT", &self.sectors_per_fat)
            .field("number of sectors per track", &self.sectors_per_track)
            .field("number of heads", &self.num_heads)
            .field("number of hidden sectors", &self.num_hidden_sectors)
            .field("total logical sectors (32bits)", &self.total_logical_sectors_32)
            .finish()
    }
}
