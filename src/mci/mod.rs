#![allow(unused)] 
#![feature(asm)]

mod regs;
mod err;
mod constants;
mod mci_config;
mod mci_timing;

use core::arch::asm;
use core::default;
use core::ptr::NonNull;
use core::time::Duration;
use log::debug;
use log::error;
use log::info;
use log::warn;
use bitflags::{bitflags, Flags};

use crate::regs::*;
use crate::set_reg32_bits;

use regs::*;
use constants::*;
use mci_timing::*;
use mci_config::*;
use err::{FsdifError, FsdifResult};

//* 核心数据结构 */
pub struct MCI {
    config: MCIConfig,
    reg: FsdifReg,
    is_ready: bool,
    prev_cmd: u32,
    curr_timing: MCITiming,
}
impl MCI {
    pub fn new(reg_base: NonNull<u8>) -> Self {
        MCI {
            reg: FsdifReg::new(reg_base),
            config: MCIConfig::new(),
            //* 暂时无脑True */
            is_ready: true,
            prev_cmd: 0,
            curr_timing: MCITiming::new(),
        }
    }

    fn read(&self, offset: u32) -> u32 {
        self.reg.read_32(offset)
    }

    ///! 可能需要修改为 read_reg 版本 
    pub fn vid(&self) -> u32 {
        self.read(FSDIF_VID_OFFSET)
    }

    pub fn uid(&self) -> u32 {
        self.read(FSDIF_UID_OFFSET)
    }

    pub fn card_detected(&self) -> bool {
        self.read(FSDIF_CARD_DETECT_OFFSET) == 0
    }

    pub fn blksize(&self) -> u32 {
        self.read(FSDIF_BLK_SIZ_OFFSET)
    }

    pub fn blksize_set(&self, blksize: u32) {
        self.reg.write_reg(FsdifBlkSiz::from_bits_truncate(blksize));
    }

    pub fn clk_freq_set(&self, clk_hz: u32) -> FsdifResult {
        let mut reg_val = FsdifCmd::UPD_CLK;
        let cmd_reg = self.reg.read_reg::<FsdifCmd>();
        let cur_cmd_index =  cmd_reg.index_get();
        if cur_cmd_index == FsDifSDIndivCommand::VoltageSwitch as u32 {
            reg_val |= FsdifCmd::VOLT_SWITCH;
        }
        if clk_hz > 0 && self.config.get_tuning as usize != default_tuning as usize {
            /* select board-related time-tuning configurations */
            let target_timing = (self.config.get_tuning)(clk_hz.into(),self.config.non_removable);
            if target_timing == MMC_SD_NULL {
                error!("No available timeing !!!");
                return Err(FsdifError::InvalidTiming);
            }
            /* update pad delay */
            
        }
        Ok(())
    }

    pub fn init_external_clk(&self) -> FsdifResult {
        let reg_val = 
        FsdifClkSrc::uhs_reg(0, 0, 0x5) | 
        FsdifClkSrc::UHS_EXT_CLK_ENA;
        if 0x502 == reg_val.bits() {
            info!("invalid uhs config"); //* 经过检查没问题 */
        }
        //? 这里可能需要抽象出一个update_external_clk的函数
        self.reg.write_reg(FsdifClkSrc::from_bits_truncate(0));
        self.reg.write_reg(reg_val);
        self.reg.wait_for(|reg|{
            (FsdifClkSts::READY & reg).bits() != 0
        },Duration::from_millis((FSDIF_TIMEOUT/100).into()),Some(100))?;
        Ok(())
    }

    /*
    trans_size: Burst size of multiple transaction;
    rx_wmark: FIFO threshold watermark level when receiving data to card.
    tx_wmark: FIFO threshold watermark level when transmitting data to card
    */
    pub fn fifoth_set(&self,
        trans_size:FsdifFifoThDmaTransSize,
        rx_wmark:u32,
        tx_wmark:u32){
        let trans_size:u32 = trans_size.into();
        let val = 
        (FsdifFifoTh::DMA_TRANS_MASK & (trans_size << 28).into()) |
        (FsdifFifoTh::RX_WMARK_MASK & (rx_wmark << 16).into()) |
        (FsdifFifoTh::TX_WMARK_MASK & tx_wmark.into());
        info!("fifoth set to 0x{:x}",val); //* 经检查无问题 */
        self.reg.write_reg(val);
    }

    /* FSDIF_CLK_SRC_OFFSET 和 FSDIF_CLKDIV_OFFSET 两个寄存器配合完成卡时钟和驱动采样相位调整
    UHS_REG_EXT 配置一级分频，CLK_DIV 决定CARD工作时钟频率, DRV 和 SAMP 分别控制驱动相位和采样相位粗调
        分配系数 = bit [14 : 8] + 1
    CLKDIV 配置二级分频, DIVIDER , DRV 和 SAMP 分别控制驱动相位和采样相位精调
        分配系数 = bit [7: 0] * 2
    */
    pub fn uhs_reg_set(&self,drv_phase: u32, samp_phase: u32, clk_div: u32) {
        self.reg.modify_reg(|reg|{
            FsdifClkSrc::uhs_reg(drv_phase, samp_phase, clk_div)}
        );
    }

    pub fn power_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifPwrEn::ENABLE | reg
            } else {
                !FsdifPwrEn::ENABLE & reg
            }
        });
        info!("power set to 0x{:x}",self.reg.read_reg::<FsdifPwrEn>());
    }

    pub fn clock_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifClkEn::CCLK_ENABLE | reg
            } else {
                !FsdifClkEn::CCLK_ENABLE & reg
            }
        });
        info!("clock set to 0x{:x}",self.reg.read_reg::<FsdifClkEn>()); //* 经检查无问题 */ */
    }

    pub fn clock_src_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifClkSrc::UHS_EXT_CLK_ENA |reg
            } else {
                !FsdifClkSrc::UHS_EXT_CLK_ENA & reg
            }
        });
        info!("clock src set to 0x{:x}",self.reg.read_reg::<FsdifClkSrc>()); //* 经检查已无误 */
    }

    pub fn voltage_1_8v_set(&self,enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifUhsReg::VOLT_180 | reg
            } else {
                !FsdifUhsReg::VOLT_180 & reg
            }
        });
        info!("voltage set to 0x{:x}",self.reg.read_reg::<FsdifUhsReg>()); //* 经检查已无误 */
    }

    pub fn bus_width_set(&self, width: u32) -> FsdifResult {
        let reg_val:FsdifCType;
        if width == 1 {
            reg_val = FsdifCType::CARD0_WIDTH2_1BIT;
        } else if width == 4 {
            reg_val = FsdifCType::CARD0_WIDTH2_4BIT;
        } else if width == 8 {
            reg_val = FsdifCType::CARD0_WIDTH1_8BIT;
        } else {
            return Err(FsdifError::NotSupport);
        }
        self.reg.write_reg(reg_val);
        info!("bus width set to 0x{:x}",self.reg.read_reg::<FsdifCType>());
        Ok(())
    }

    pub fn ctrl_reset(&self, reset_bits: FsdifCtrl) -> FsdifResult {
        self.reg.modify_reg(|reg| {
            reset_bits | reg
        });
        self.reg.wait_for(|reg| {
            (reset_bits & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        /* update clock after reset */
        self.send_private_cmd(FsdifCmd::UPD_CLK, 0)?;
        /* for fifo reset, need to check if fifo empty */
        if reset_bits.contains(FsdifCtrl::FIFO_RESET) {
            self.reg.wait_for(|reg| {
                (FsdifStatus::FIFO_EMPTY & reg).bits() != 0
            }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        }
        Ok(())
    }

    pub fn send_private_cmd(&self, cmd:FsdifCmd, arg: u32) -> FsdifResult {
        self.reg.wait_for(|reg| {
            (FsdifStatus::DATA_BUSY & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        self.reg.write_reg(FsdifCmdArg::from_bits_truncate(arg));
        // unsafe { dsb() };/* drain writebuffer */
        self.reg.write_reg(FsdifCmd::START | cmd);
        self.reg.wait_for(|reg|{
            (FsdifCmd::START & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn descriptor_set(&self, desc: u32) {
        self.reg.write_reg(FsdifDescListAddrH::empty());
        self.reg.write_reg(FsdifDescListAddrL::from_bits_truncate(desc));
    }

    pub fn idma_reset(&self) {
        let mut reg_val = self.reg.read_reg::<FsdifBusMode>();
        reg_val |= FsdifBusMode::SWR;
        self.reg.write_reg(reg_val);
    }

    pub fn poll_wait_busy_card(&self) -> FsdifResult {
        let busy_bits = FsdifStatus::DATA_BUSY | FsdifStatus::DATA_STATE_MC_BUSY;
        let reg_val = self.reg.read_reg::<FsdifStatus>();
        if reg_val.contains(busy_bits.clone()) {
           warn!("Card is busy, waiting ...");
        }
        self.reg.wait_for(|reg|{
            (busy_bits & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn trans_bytes_set(&self, bytes: u32) {
        self.reg.write_reg(FsdifBytCnt::from_bits_truncate(bytes));
    }

    pub fn raw_status_get(&self) -> FsdifRawInts{
        return self.reg.read_reg::<FsdifRawInts>();
    }

    pub fn raw_status_clear(&self) {
        let reg_val = self.raw_status_get();
        self.reg.write_reg(reg_val);
    }

    pub fn busy_card_reset(&self) -> FsdifResult {
        self.reg.modify_reg(|reg| {
            FsdifCtrl::CONTROLLER_RESET | reg
        });
        self.reg.wait_for(|reg|{
            self.reg.modify_reg(|reg| {
                FsdifCtrl::CONTROLLER_RESET | reg
            });
            (FsdifStatus::DATA_BUSY & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn reset(&self) -> FsdifResult {
        /* check FsdifCtrl */
        info!("FsdifCtrl: 0x{:x}",self.reg.read_reg::<FsdifCtrl>());
        /* set fifo */
        self.fifoth_set(
            FsdifFifoThDmaTransSize::DmaTrans8, 
            FSDIF_RX_WMARK, 
            FSDIF_TX_WMARK); //* 经检查已无误 */
        debug!("fifoth set success");
        /* set card threshold */
        self.reg.write_reg( 
            FsdifCardThrctl::CARDRD |
            FsdifFifoDepth::Depth8.card_thrctl_threshold().into());
        info!("card threshold set to 0x{:x}",
        (FsdifCardThrctl::CARDRD |
        FsdifFifoDepth::Depth8.card_thrctl_threshold().into()).bits()); //* 经检查已无误 */
        debug!("card threshold set success");
        /* disable clock and update ext clk */
        self.clock_set(false); //* 经检查已无误 */
        debug!("clock set success");
        /* set 1st clock */
        self.init_external_clk()?; //* 经检查已无误 */
        debug!("external clock init success");
        /* power on */
        self.power_set(true); //* 经检查已无误 */
        self.clock_set(true); //* 经检查已无误 */
        self.clock_src_set(true); //* 经检查已无误 */
        debug!("power on success");
        /* set voltage as 3.3v */
        self.voltage_1_8v_set(false); //* 经检查已无误 */
        debug!("voltage set to 3.3v");
        /* set bus width as 1 */
        self.bus_width_set(1)?; //* 经检查已无误 */
        debug!("bus width set to 1");
        /* reset controller and card */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET | FsdifCtrl::DMA_RESET)?;
        } else {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET)?; //* 经检查已无误 */
        }
        debug!("controller reset success");
        /* send private command to update clock */
        self.send_private_cmd(FsdifCmd::UPD_CLK, 0)?; //? 暂时不知道怎么检查 ?/
        debug!("send private command success");
        /* reset card for no-removeable media, e.g. eMMC */
        if self.config.non_removable {
            self.reg.modify_reg(|reg|{
                FsdifCardReset::ENABLE | reg
            });
        }else {
            self.reg.modify_reg(|reg|{
                !FsdifCardReset::ENABLE & reg
            });
        }
        info!("card reset to 0x{:x}",self.reg.read_reg::<FsdifCardReset>());
        debug!("card reset success");
        /* clear interrupt status */ //* 经检查已无误 */
        self.reg.write_reg(FsdifInt::empty());
        info!("clear interrupt status to 0x{:x}",self.reg.read_reg::<FsdifInt>());
        let reg_val = self.reg.read_reg::<FsdifRawInts>();
        self.reg.write_reg(reg_val);
        info!("clear interrupt status to 0x{:x}",self.reg.read_reg::<FsdifRawInts>());
        self.reg.write_reg(FsdifDmacIntEn::empty());
        info!("clear interrupt status to 0x{:x}",self.reg.read_reg::<FsdifDmacIntEn>());
        let reg_val = self.reg.read_reg::<FsdifDmacStatus>();
        self.reg.write_reg(reg_val);
        info!("clear interrupt status to 0x{:x}",self.reg.read_reg::<FsdifDmacStatus>());
        debug!("clear interrupt status success");
        /* enable card detect interrupt */
        if !self.config.non_removable {
            self.reg.modify_reg(|reg|{
                FsdifInt::CD_BIT | reg
            });
        }
        info!("enable card detect interrupt to 0x{:x}",self.reg.read_reg::<FsdifInt>());
        debug!("enable card detect interrupt success");
        /* enable controller and internal DMA */
        self.reg.modify_reg(|reg|{
            FsdifCtrl::INT_ENABLE | FsdifCtrl::USE_INTERNAL_DMAC | reg
        });
        info!("enable controller and internal DMA to 0x{:x}",self.reg.read_reg::<FsdifCtrl>()); //* 经检查已无误 */
        debug!("enable controller and internal DMA success");
        /* set data and resp timeout */
        self.reg.write_reg(FsdifTimeout::timeout_data(
            FsdifTimeout::MAX_DATA_TIMEOUT, 
            FsdifTimeout::MAX_RESP_TIMEOUT));
        info!("set data and resp timeout to 0x{:x}",self.reg.read_reg::<FsdifTimeout>());
        debug!("set data and resp timeout success");
        /* reset descriptors and dma */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.descriptor_set(0);
            self.idma_reset();
        }
        debug!("reset descriptors and dma success");
        Ok(())
    }
}

//* CMD 相关的函数 */
impl MCI {
    pub fn block_size_set(&self, blksize: u32) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SetBlockLength as u32,
            cmdarg: blksize,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP,
            data: FsdifBuf {
                buf: &mut [],
                buf_dma: 0,
                blksz: 0,
                blkcnt: 0,
            },
            success: false,
        };
        self.pio_transfer(&cmd_data)?;
        self.poll_wait_pio_end(&mut cmd_data)?;
        Ok(())
    }
    pub fn block_count_set(&self, blkcnt: u32) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SetBlockCount as u32,
            cmdarg: blkcnt,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP,
            data: FsdifBuf {
                buf: &mut [],
                buf_dma: 0,
                blksz: 0,
                blkcnt: 0,
            },
            success: false,
        };
        self.pio_transfer(&cmd_data)?;
        self.poll_wait_pio_end(&mut cmd_data)?;
        Ok(())
    }

}

//* PIO 相关的函数 */
impl MCI {
    pub fn pio_write_data<'a>(&self, data: &[u32]) -> FsdifResult {
        self.reg.write_reg(FsdifCmd::DAT_WRITE);
        for i in 0..data.len() {
            self.reg.write_reg(FsdifData::from_bits_truncate(data[i]));
        }
        Ok(())
    }

    pub fn pio_read_data(&self, data: &mut [u32]) -> FsdifResult {
        if data.len() > FSDIF_MAX_FIFO_CNT as usize {
            return Err(FsdifError::NotSupport);
        }
        for i in 0..data.len() {
            data[i] = self.reg.read_reg::<FsdifData>().bits();
        }
        Ok(())
    }

    pub fn pio_transfer(&self, cmd_data: &FSdifCmdData) -> FsdifResult {
        let read = cmd_data.flag.contains(CmdFlag::READ_DATA);
        //? 实验性代码
        /* enable related interrupt */
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr, FsdifInt::INTS_DATA_MASK.bits(), true);
        //? 实验性代码
        if !self.is_ready{
            error!("device is not yet initialized!!!");
            return Err(FsdifError::NotInit);
        }
        if self.config.trans_mode != FsDifTransMode::PioTransMode {
            return Err(FsdifError::InvalidState);
        }
        /* for removable media, check if card exists */
        if !self.config.non_removable && !self.card_detected() {
            error!("card is not detected !!!");
            return Err(FsdifError::NoCard);
        }
        /* wait previous command finished and card not busy */
        self.poll_wait_busy_card()?;
        /* reset fifo and not use DMA */
        self.reg.modify_reg(|reg|{
            !FsdifCtrl::USE_INTERNAL_DMAC & reg
        });
        self.ctrl_reset(FsdifCtrl::FIFO_RESET)?;
        self.reg.modify_reg(|reg|{
            !FsdifBusMode::DE & reg
        });
        /* write data */
        if cmd_data.data.buf.len() > 0 {
            /* while in PIO mode, max data transferred is 0x800 */
            if cmd_data.data.buf.len() > FSDIF_MAX_FIFO_CNT as usize {
                error!("Fifo do not support writing more than {:x}.",FSDIF_MAX_FIFO_CNT);
                return Err(FsdifError::NotSupport);
            }
            /* set transfer data length and block size */
            self.trans_bytes_set(cmd_data.data.buf.len() as u32);
            self.blksize_set(cmd_data.data.blksz);
            /* if need to write, write to fifo before send command */
            if !read { 
                /* invalide buffer for data to write */
                // unsafe { dsb() };
                self.pio_write_data(cmd_data.data.buf)?;
            }
        }
        self.transfer_cmd(cmd_data)?;
        Ok(())
    }

    pub fn poll_wait_pio_end(&self,cmd_data: &mut FSdifCmdData) -> FsdifResult{
        let read = cmd_data.flag.contains(CmdFlag::READ_DATA);
        if !self.is_ready {
            error!("device is not yet initialized!!!");
            return Err(FsdifError::NotInit);
        }
        if FsDifTransMode::PioTransMode != self.config.trans_mode {
            error!("device is not configure in PIO transfer mode.");
            return Err(FsdifError::InvalidState);
        }
        info!("wait for PIO cmd to finish ...");
        self.reg.wait_for(|reg|{
            (FsdifRawInts::CMD_BIT & reg).bits() != 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        /* if need to read data, read fifo after send command */
        if read {
            info!("wait for PIO data to read ...");
            self.reg.wait_for(|reg|{
                (FsdifRawInts::DTO_BIT & reg).bits() != 0
            }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
            /* clear status to ack */
            self.raw_status_clear();
            info!("card cnt: 0x{:x}, fifo cnt: 0x{:x}",
            self.reg.read_reg::<FsdifTranCardCnt>(),
            self.reg.read_reg::<FsdifTranFifoCnt>());
        }
        /* clear status to ack cmd done */
        self.raw_status_clear();
        self.get_cmd_response(cmd_data);
        Ok(())
    }
}
pub struct FsdifBuf<'a> {
    pub buf: &'a mut [u32],
    pub buf_dma: u32,
    pub blksz: u32,
    pub blkcnt: u32,
}
pub struct FSdifCmdData<'a> {
    pub cmdidx: u32,
    pub cmdarg: u32,
    pub response: [u32; 4],
    pub flag: CmdFlag,
    pub data: FsdifBuf<'a>,
    pub success: bool
}

bitflags! {
    pub struct CmdFlag: u32 {
        const NEED_INIT = 0x1;
        const EXP_RESP = 0x2;
        const EXP_LONG_RESP = 0x4;
        const NEED_RESP_CRC = 0x8;
        const EXP_DATA = 0x10;
        const WRITE_DATA = 0x20;
        const READ_DATA = 0x40;
        const NEED_AUTO_STOP = 0x80;
        const ADTC = 0x100;
        const SWITCH_VOLTAGE = 0x200;
        const ABORT = 0x400;
        const AUTO_CMD12 = 0x800;
    }
}

//* CMD 相关的方法 */
impl MCI {
    const FSDIF_EXT_APP_CMD: u32 = 55;
    const FSDIF_SWITCH_VOLTAGE:u32 = 11;
    pub fn transfer_cmd(&self, cmd_data: &FSdifCmdData) -> FsdifResult {
        let mut raw_cmd = FsdifCmd::empty();
        if self.curr_timing.use_hold {
            raw_cmd |= FsdifCmd::USE_HOLD_REG;
        }
        if cmd_data.flag.contains(CmdFlag::ABORT) {
            raw_cmd |= FsdifCmd::STOP_ABORT;
        }
        /* 命令需要进行卡初始化，如CMD-0 */
        if cmd_data.flag.contains(CmdFlag::NEED_INIT) {
            raw_cmd |= FsdifCmd::INIT;
        }
        /* 命令涉及电压切换 */
        if cmd_data.flag.contains(CmdFlag::SWITCH_VOLTAGE) {
            raw_cmd |= FsdifCmd::VOLT_SWITCH;
        }
        /* 命令传输过程伴随数据传输 */
        if cmd_data.flag.contains(CmdFlag::EXP_DATA) {
            raw_cmd |= FsdifCmd::DAT_EXP;
            if cmd_data.flag.contains(CmdFlag::WRITE_DATA) {
                raw_cmd |= FsdifCmd::DAT_WRITE;
            }
        }
        /* 命令需要进行CRC校验 */
        if cmd_data.flag.contains(CmdFlag::NEED_RESP_CRC) {
            raw_cmd |= FsdifCmd::RESP_CRC;
        }
        /* 命令需要响应回复 */
        if cmd_data.flag.contains(CmdFlag::EXP_RESP) {
            raw_cmd |= FsdifCmd::RESP_EXP;
            if cmd_data.flag.contains(CmdFlag::EXP_LONG_RESP) {
                raw_cmd |= FsdifCmd::RESP_LONG;
            }
        }
        raw_cmd |= FsdifCmd::from_bits_truncate(set_reg32_bits!(cmd_data.cmdidx, 5, 0));
        debug!("============[{}-{}]@0x{:x} begin ============",
        {
            if self.prev_cmd == Self::FSDIF_EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx,
        self.reg.addr.as_ptr() as usize );
        debug!("    cmd: 0x{:x}", raw_cmd.bits());
        debug!("    arg: 0x{:x}", cmd_data.cmdarg);
        /* enable related interrupt */
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr, FsdifInt::INTS_CMD_MASK.bits(), true);
        self.send_private_cmd(raw_cmd, cmd_data.cmdarg);
        info!("cmd send done");
        Ok(())
    }
    pub fn get_cmd_response(&self,cmd_data: &mut FSdifCmdData) -> FsdifResult{
        let read = cmd_data.flag.contains(CmdFlag::READ_DATA);
        if !self.is_ready {
            error!("device is not yet initialized!!!");
            return Err(FsdifError::NotInit);
        }
        if read {
            if FsDifTransMode::PioTransMode == self.config.trans_mode {
                self.pio_read_data(cmd_data.data.buf)?;
            }
        }
        /* check response of cmd */
        if cmd_data.flag.contains(CmdFlag::EXP_RESP) {
            if cmd_data.flag.contains(CmdFlag::EXP_LONG_RESP) {
                cmd_data.response[0] = self.reg.read_reg::<FsdifResp0>().bits();
                cmd_data.response[1] = self.reg.read_reg::<FsdifResp1>().bits();
                cmd_data.response[2] = self.reg.read_reg::<FsdifResp2>().bits();
                cmd_data.response[3] = self.reg.read_reg::<FsdifResp3>().bits();
                debug!("    resp: 0x{:x}-0x{:x}-0x{:x}-0x{:x}",
                cmd_data.response[0],cmd_data.response[1],cmd_data.response[2],cmd_data.response[3]);
            }else {
                cmd_data.response[0] = self.reg.read_reg::<FsdifResp0>().bits();
                cmd_data.response[1] = 0;
                cmd_data.response[2] = 0;
                cmd_data.response[3] = 0;
                debug!("    resp: 0x{:x}",cmd_data.response[0]);
            }
        }
        cmd_data.success = true;
        debug!("============[{}-{}]@0x{:x} begin ============",
        {
            if self.prev_cmd == Self::FSDIF_EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx,
        self.reg.addr.as_ptr() as usize );
        /* disable related interrupt */
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr,(FsdifInt::INTS_CMD_MASK|FsdifInt::INTS_DATA_MASK).bits(),false);
        self.interrupt_mask_set(FsDifIntrType::DmaIntr,FsdifDmacIntEn::INTS_MASK.bits(),false);
        info!("cmd send done ...");
        Ok(())
    }
}

//* Interrupt 相关的方法 */
impl MCI {
    pub fn interrupt_mask_get(&self, tp: FsDifIntrType) -> u32 {
        let mut mask = 0;
        if FsDifIntrType::GeneralIntr == tp {
            mask = self.reg.read_reg::<FsdifInt>().bits();
        } else if FsDifIntrType::DmaIntr == tp {
            mask = self.reg.read_reg::<FsdifDmacIntEn>().bits();
        }
        //? 这里不知道要不要用Some作为返回值 
        mask
    }

    pub fn interrupt_mask_set(&self, tp: FsDifIntrType, set_mask: u32, enable: bool) {
        info!("set_mask 0x{:x}",set_mask);
        info!("enable {}",enable);
        let mut mask = self.interrupt_mask_get(tp);
        if enable {
            mask |= set_mask;
        } else {
            mask &= !set_mask;
        }
        if FsDifIntrType::GeneralIntr == tp {
            self.reg.write_reg(FsdifInt::from_bits_truncate(mask));
        } else if FsDifIntrType::DmaIntr == tp {
            self.reg.write_reg(FsdifDmacIntEn::from_bits_truncate(mask));
        }
        info!("FsdifInt 0x{:x}",self.reg.read_reg::<FsdifInt>());
    }
}

impl MCI {
    pub fn dump_register(&self) {
        warn!("cntrl: 0x{:x}", self.reg.read_reg::<FsdifCtrl>());
        warn!("pwren: 0x{:x}", self.reg.read_reg::<FsdifPwrEn>());
        warn!("clkdiv: 0x{:x}", self.reg.read_reg::<FsdifClkDiv>());
        warn!("clkena: 0x{:x}", self.reg.read_reg::<FsdifClkEn>());
        warn!("tmout: 0x{:x}", self.reg.read_reg::<FsdifTimeout>());
        warn!("ctype: 0x{:x}", self.reg.read_reg::<FsdifCType>());
        warn!("blksz: 0x{:x}", self.reg.read_reg::<FsdifBlkSiz>());
        warn!("blkcnt: 0x{:x}", self.reg.read_reg::<FsdifBytCnt>());
        warn!("intmask: 0x{:x}", self.reg.read_reg::<FsdifInt>());
        warn!("cmdarg: 0x{:x}", self.reg.read_reg::<FsdifCmdArg>());
        warn!("cmd: 0x{:x}", self.reg.read_reg::<FsdifCmd>());
        warn!("resp0: 0x{:x}", self.reg.read_reg::<FsdifResp0>());
        warn!("reps1: 0x{:x}", self.reg.read_reg::<FsdifResp1>());
        warn!("resp2: 0x{:x}", self.reg.read_reg::<FsdifResp2>());
        warn!("resp3: 0x{:x}", self.reg.read_reg::<FsdifResp3>());
        warn!("maskints: 0x{:x}", self.reg.read_reg::<FsdifMaskedInts>());
        warn!("rawints: 0x{:x}", self.reg.read_reg::<FsdifRawInts>());
        warn!("status: 0x{:x}", self.reg.read_reg::<FsdifStatus>());
        warn!("fifoth: 0x{:x}", self.reg.read_reg::<FsdifFifoTh>());
        warn!("carddet: 0x{:x}", self.reg.read_reg::<FsdifCardDetect>());
        warn!("wrtprt: 0x{:x}", self.reg.read_reg::<FsdifCardWrtp>());
        warn!("cksts: 0x{:x}", self.reg.read_reg::<FsdifClkSts>());
        warn!("trans_cardcnt: 0x{:x}", self.reg.read_reg::<FsdifTranCardCnt>());
        warn!("trans_fifocnt: 0x{:x}", self.reg.read_reg::<FsdifTranFifoCnt>());
        warn!("debnce: 0x{:x}", self.reg.read_reg::<FsdifDebnce>());
        warn!("uid: 0x{:x}", self.reg.read_reg::<FsdifUid>());
        warn!("vid: 0x{:x}", self.reg.read_reg::<FsdifVid>());
        warn!("hwconf: 0x{:x}", self.reg.read_reg::<FsdifHwconf>());
        warn!("uhsreg: 0x{:x}", self.reg.read_reg::<FsdifUhsReg>());
        warn!("cardreset: 0x{:x}", self.reg.read_reg::<FsdifCardReset>());
        warn!("busmode: 0x{:x}", self.reg.read_reg::<FsdifBusMode>());
        warn!("descaddrl: 0x{:x}", self.reg.read_reg::<FsdifDescListAddrL>());
        warn!("descaddrh: 0x{:x}", self.reg.read_reg::<FsdifDescListAddrH>());
        warn!("dmacstatus: 0x{:x}", self.reg.read_reg::<FsdifDmacStatus>());
        warn!("dmacinten: 0x{:x}", self.reg.read_reg::<FsdifDmacIntEn>());
        warn!("curdescaddrl: 0x{:x}", self.reg.read_reg::<FsdifCurDescAddrL>());
        warn!("curdescaddrh: 0x{:x}", self.reg.read_reg::<FsdifCurDescAddrH>());
        warn!("curbufaddrl: 0x{:x}", self.reg.read_reg::<FsdifCurBufAddrL>());
        warn!("curbufaddrh: 0x{:x}", self.reg.read_reg::<FsdifCurBufAddrH>());
        warn!("card_thrctl: 0x{:x}", self.reg.read_reg::<FsdifCardThrctl>());
        warn!("clock_src: 0x{:x}", self.reg.read_reg::<FsdifClkSrc>());
        warn!("emmcddr: 0x{:x}", self.reg.read_reg::<FsdifEmmcDdrReg>());
        warn!("enableshift: 0x{:x}", self.reg.read_reg::<FsdifEnableShift>());

    }
}

// 定义传输模式枚举
#[derive(Debug, PartialEq)]
pub enum FsDifTransMode {
    DmaTransMode,      // DMA传输模式
    PioTransMode,      // PIO传输模式（通过读/写Fifo）
}

// 定义中断类型枚举
#[derive(Debug, PartialEq)]
#[derive(Clone, Copy)]
pub enum FsDifIntrType {
    GeneralIntr,       // 属于控制器的中断状态
    DmaIntr,           // 属于DMA的中断状态
}

// 定义事件类型枚举
#[derive(Debug, PartialEq)]
pub enum FsDifEvtType {
    CardDetected = 0,  // 卡检测事件
    CmdDone,           // 命令传输完成事件
    DataDone,          // 包含数据的命令传输完成事件
    SdioIrq,           // SDIO卡自定义事件
    ErrOccured,        // 传输中出现错误
    NumOfEvt,          // 事件数量
}

// 定义时钟速度枚举
#[derive(Debug, PartialEq)]
pub enum FsDifClkSpeed {
    ClkSpeedClose =  0,
    ClkSpeed400KHz = 400_000,
    ClkSpeed25Mhz = 25_000_000,
    ClkSpeed26Mhz = 26_000_000, // mmc
    ClkSpeed50Mhz = 50_000_000,
    ClkSpeed52Mhz = 52_000_000, // mmc
    ClkSpeed66Mhz = 66_000_000, // mmc
    ClkSpeed100Mhz = 100_000_000,
}

impl From<u32> for FsDifClkSpeed {
    fn from(value: u32) -> Self {
        match value {
            400_000 => FsDifClkSpeed::ClkSpeed400KHz,
            25_000_000 => FsDifClkSpeed::ClkSpeed25Mhz,
            26_000_000 => FsDifClkSpeed::ClkSpeed26Mhz,
            50_000_000 => FsDifClkSpeed::ClkSpeed50Mhz,
            52_000_000 => FsDifClkSpeed::ClkSpeed52Mhz,
            66_000_000 => FsDifClkSpeed::ClkSpeed66Mhz,
            100_000_000 => FsDifClkSpeed::ClkSpeed100Mhz,
            _ => FsDifClkSpeed::ClkSpeedClose,
        }
    }
}

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}

pub enum FsDifCommand {
    GoIdleState        = 0,  /*< Go Idle State */
    AllSendCid         = 2,  /*< All Send CID */
    SetDsr             = 4,  /*< Set DSR */
    SelectCard         = 7,  /*< Select Card */
    SendCsd            = 9,  /*< Send CSD */
    SendCid            = 10, /*< Send CID */
    StopTransmission   = 12, /*< Stop Transmission */
    SendStatus         = 13, /*< Send Status */
    GoInactiveState    = 15, /*< Go Inactive State */
    SetBlockLength     = 16, /*< Set Block Length */
    ReadSingleBlock    = 17, /*< Read Single Block */
    ReadMultipleBlock  = 18, /*< Read Multiple Block */
    SetBlockCount      = 23, /*< Set Block Count */
    WriteSingleBlock   = 24, /*< Write Single Block */
    WriteMultipleBlock = 25, /*< Write Multiple Block */
    ProgramCsd         = 27, /*< Program CSD */
    SetWriteProtect    = 28, /*< Set Write Protect */
    ClearWriteProtect  = 29, /*< Clear Write Protect */
    SendWriteProtect   = 30, /*< Send Write Protect */
    Erase              = 38, /*< Erase */
    LockUnlock         = 42, /*< Lock Unlock */
    ApplicationCommand = 55, /*< Send Application Command */
    GeneralCommand     = 56, /*< General Purpose Command */
    ReadOcr            = 58, /*< Read OCR */
}

pub enum FsDifSDIndivCommand {
    SendRelativeAddress    = 3,  /*< Send Relative Address */
    Switch                 = 6,  /*< Switch Function */
    SendInterfaceCondition = 8,  /*< Send Interface Condition */
    VoltageSwitch          = 11, /*< Voltage Switch */
    SpeedClassControl      = 20, /*< Speed Class control */
    EraseWriteBlockStart   = 32, /*< Write Block Start */
    EraseWriteBlockEnd     = 33, /*< Write Block End */
    SendTuningBlock        = 19, /*< Send Tuning Block */
}