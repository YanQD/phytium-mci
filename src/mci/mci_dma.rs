use alloc::vec::Vec;
use dma_api::DSlice;
use log::*;

use super::mci_data::MCIData;
use super::MCI;
use super::err::*;
use super::constants::*;
use super::regs::*;

#[derive(Default)]
pub struct FSdifIDmaDesc {
    pub attribute: u32,
    pub non1: u32,
    pub len: u32,
    pub non2: u32,
    pub addr_lo: u32,
    pub addr_hi: u32,
    pub desc_lo: u32,
    pub desc_hi: u32,
}

pub struct FSdifIDmaDescList {
    pub first_desc: *mut FSdifIDmaDesc,
    pub first_desc_dma: usize,  // 第一个descriptor的物理地址
    pub desc_num: u32,
    pub desc_trans_sz: u32,   // 单个descriptor传输的字节数
}

impl FSdifIDmaDescList {
    pub fn new() -> Self {
        FSdifIDmaDescList {
            first_desc: core::ptr::null_mut(),
            first_desc_dma: 0,
            desc_num: 0,
            desc_trans_sz: 0,
        }
    }
}

//* DMA 相关的函数 */
impl MCI {
    pub fn dma_int_set(&mut self) {
        self.config.reg().modify_reg(|reg| {
            MCIDMACIntEn::RI | MCIDMACIntEn::TI | MCIDMACIntEn::FBE | reg
        });
    }

    pub fn dump_dma_descriptor(&self, desc_in_use: u32) {
        debug!("{} dma desc in use!", desc_in_use);
        if !self.desc_list.first_desc.is_null() {
            for i in 0..desc_in_use {
                unsafe {
                    let cur_desc = &*self.desc_list.first_desc.add(i as usize);
                    debug!("descriptor no {} @{:p}", i, cur_desc);
                    debug!("\tattribute: 0x{:x}", (*cur_desc).attribute);
                    debug!("\tnon1: 0x{:x}", (*cur_desc).non1);
                    debug!("\tlen: 0x{:x}", (*cur_desc).len);
                    debug!("\tnon2: 0x{:x}", (*cur_desc).non2);
                    debug!("\taddr_lo: 0x{:x}", (*cur_desc).addr_lo);
                    debug!("\taddr_hi: 0x{:x}", (*cur_desc).addr_hi);
                    debug!("\tdesc_lo: 0x{:x}", (*cur_desc).desc_lo);
                    debug!("\tdesc_hi: 0x{:x}", (*cur_desc).desc_hi);
                }
            }
        }
        debug!("dump ok");
    }

    /// setup DMA descriptor list before do transcation
    pub(crate) fn setup_dma_descriptor(&mut self, data: &MCIData) -> MCIResult {
        let desc_list = &self.desc_list;
        // 一个desc可以传输的块数
        let desc_blocks = desc_list.desc_trans_sz / data.blksz(); 
        let mut remain_blocks = data.blkcnt();
        let mut buf_addr = data.buf_dma();
        let mut trans_blocks: u32; // 本次循环被传输的块
        let mut is_first;
        let mut is_last;

        let mut desc_num = 1u32;
        let data_len = data.blkcnt() * data.blksz();
        // 计算需要多少desc来传输
        if data_len > desc_list.desc_trans_sz {
            desc_num = data_len / desc_list.desc_trans_sz;
            desc_num += if data_len % desc_list.desc_trans_sz == 0 { 0 } else { 1 };
        }

        if desc_num > desc_list.desc_num {
            error!("Transfer descriptor are not enough! desc need: {}, desc available: {}", desc_num, desc_list.desc_num);
            return Err(MCIError::ShortBuf);
        }

        debug!("DMA transfer 0x{:x} use {} desc, total {} available", data.buf_dma(), desc_num, desc_list.desc_num);

        // setup DMA descriptor list, so that we just need to update buffer address in each transcation
        let total_size = desc_list.desc_num as usize * core::mem::size_of::<FSdifIDmaDesc>();
        unsafe {
            core::ptr::write_bytes(desc_list.first_desc as *mut u8, 0, total_size);
        }

        for i in 0..desc_num {
            trans_blocks = if remain_blocks <= desc_blocks { remain_blocks } else { desc_blocks };
            unsafe {
                let cur_desc = self.desc_list.first_desc.add(i as usize);
                let mut next_desc_addr = desc_list.first_desc_dma + (i + 1) as usize * core::mem::size_of::<FSdifIDmaDesc>();
    
                is_first = i == 0;
                is_last = desc_num - 1 == i;

                // set properity of descriptor entry
                (*cur_desc).attribute = FSDIF_IDMAC_DES0_CH | FSDIF_IDMAC_DES0_OWN;
                if is_first { (*cur_desc).attribute |= FSDIF_IDMAC_DES0_FD; }
                if is_last { (*cur_desc).attribute |= FSDIF_IDMAC_DES0_LD | FSDIF_IDMAC_DES0_ER; }

                // set data length in transfer
                (*cur_desc).non1 = 0u32;
                (*cur_desc).len = trans_blocks * data.blksz();

                // set data buffer for transfer
                if buf_addr % data.blksz() as usize != 0 {
                    error!("Data buffer 0x{:x} do not align to {}!", buf_addr, data.blksz());
                    return Err(MCIError::DmaBufUnalign);
                }

                if cfg!(target_arch = "aarch64") {
                    (*cur_desc).addr_hi = ((buf_addr >> 32) & 0xFFFF_FFFF) as u32;
                    (*cur_desc).addr_lo = (buf_addr & 0xFFFF_FFFF) as u32;
                } else {
                    (*cur_desc).addr_hi = 0;
                    (*cur_desc).addr_lo = (buf_addr & 0xFFFF_FFFF) as u32;
                }

                // set address of next descriptor entry, NULL for last entry
                next_desc_addr = if is_last { 0 } else { next_desc_addr };
                if next_desc_addr as usize % core::mem::size_of::<FSdifIDmaDesc>() != 0 { // make sure descriptor aligned and not cross page boundary
                    error!("DMA descriptor 0x{:x} do not align!", next_desc_addr);
                    return Err(MCIError::DmaBufUnalign);
                }

                if cfg!(target_arch = "aarch64") {
                    (*cur_desc).desc_hi = ((next_desc_addr >> 32) & 0xFFFF_FFFF) as u32;
                    (*cur_desc).desc_lo = (next_desc_addr & 0xFFFF_FFFF) as u32;
                } else {
                    (*cur_desc).desc_hi = 0;
                    (*cur_desc).desc_lo = (next_desc_addr & 0xFFFF_FFFF) as u32;
                }

                buf_addr += (*cur_desc).len as usize;
                remain_blocks -= trans_blocks;
            }
        }

        // todo 不太优雅 考虑后续修改
        let desc_vec = unsafe {
            core::mem::ManuallyDrop::new(
                Vec::from_raw_parts(desc_list.first_desc, desc_num as usize, desc_num as usize)
            )
        };
        let _ = DSlice::from(&desc_vec[..]);
        // unsafe { dsb(); }
        self.dump_dma_descriptor(desc_num);
        debug!("set dma desc ok");

        Ok(())
    }

    /// start DMA transfers for data
    #[cfg(feature = "dma")]
    pub(crate) fn dma_transfer_data(&mut self, data: &MCIData) -> MCIResult {
        self.interrupt_mask_set(MCIIntrType::GeneralIntr, MCIIntMask::INTS_DATA_MASK.bits(), true);
        self.interrupt_mask_set(MCIIntrType::DmaIntr, MCIDMACIntEn::INTS_MASK.bits(), true);

        self.setup_dma_descriptor(&data)?;

        let data_len = data.blkcnt() * data.blksz();
        info!("Descriptor@{:p}, trans bytes: {}, block size: {}", self.desc_list.first_desc, data_len, data.blksz());

        self.descriptor_set(self.desc_list.first_desc_dma);
        self.trans_bytes_set(data_len);
        self.blksize_set(data.blksz());

        Ok(())
    }
}