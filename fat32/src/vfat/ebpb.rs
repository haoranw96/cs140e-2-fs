use std::fmt;
use std::{mem};

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    /* BPB */
    pub jump_short_nop: [u8; 3],
    pub oem_id: [u8; 8],
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
    pub fat_version: [u8;2],
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
    pub boot_code: [u8; 420],
    pub bootable_signature: u16, 
}

impl BiosParameterBlock {
    fn modify_byte_order(bpb: BiosParameterBlock) -> BiosParameterBlock {
        // Too lazy to implement, should have no problem on pi
        bpb
    }

    pub fn sectors_per_fat(&self) -> u32 {
        if self.sectors_per_fat != 0 {
            self.sectors_per_fat as u32
        } else {
            self.sectors_per_fat_32
        }
    }

    pub fn total_logical_sectors(&self) -> u32 {
        if self.total_logical_sectors != 0 {
            self.total_logical_sectors as u32
        } else {
            self.total_logical_sectors_32
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
        let mut bpb_buf = [0u8; mem::size_of::<BiosParameterBlock>()];
        if let Err(e) = device.read_sector(sector, &mut bpb_buf) {
            return Err(Error::Io(e));
        }
        let bpb : BiosParameterBlock = unsafe{ mem::transmute(bpb_buf) };

        if bpb.bootable_signature != 0xAA55 {
            return Err(Error::BadSignature);
        }

        Ok(bpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock")
         .field("oem id", &String::from_utf8_lossy(&self.oem_id))
         .field("bytes per sector", &self.bytes_per_sector)
         .field("sectors per cluster", &self.sectors_per_cluster)
         .field("number of reserved sectors", &self.num_reserved_sectors)
         .field("number of FAT", &self.num_fat)
//         .field("maximum directory entries", &self.max_dir_entries)
         .field("total logical sectors", &self.total_logical_sectors())
         .field("media description type", &format!("0x{:X}", &self.media_desc_type))
         .field("number of sectors per FAT", &self.sectors_per_fat())
         .field("number of sectors per track", &self.sectors_per_track)
         .field("number of heads", &self.num_heads)
         .field("number of hidden sectors", &self.num_hidden_sectors)
         .field("flags", &self.flags)
         .field("fat_version", &self.fat_version)
         .field("root_cluster", &self.root_cluster)
         .field("fsinfo_sector", &self.fsinfo_sector)
         .field("backup_boot_sector", &self.backup_boot_sector)
         .field("drive_num", &format!("0x{:X}", &self.drive_num))
         .field("win_nt_flag", &self.win_nt_flag)
         .field("signature", &format!("0x{:X}", &self.signature))
         .field("volumn_id", &format!("0x{:X}", &self.volumn_id))
         .field("volumn_label", &String::from_utf8_lossy(&self.volumn_label))
         .field("sys_id_str", &String::from_utf8_lossy(&self.sys_id_str))
         .field("bootable_signature", &format!("0x{:X}",&self.bootable_signature))
         .finish()
    }
}
