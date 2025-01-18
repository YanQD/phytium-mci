#![allow(unused)] 
#![feature(asm)]
use core::arch::asm;
use core::ptr::NonNull;
use core::time::Duration;
use log::debug;
use log::error;
use log::info;
use log::warn;
use bitflags::{bitflags, Flags};

use crate::constants::*;
use crate::regs::*;
use crate::err::{FsdifError, FsdifResult};
use crate::set_reg32_bits;

//* 辅助数据结构 */
pub struct MCIConfig {
    trans_mode: FsDifTransMode, /* Trans mode, PIO/DMA */
    non_removable: bool,        /* Non-removable media, e.g. eMMC */
}

impl MCIConfig {
    pub fn new() -> Self {
        MCIConfig {
            trans_mode: FsDifTransMode::PioTransMode,
            non_removable: false,
        }
    }
}

pub struct MCITiming {
    use_hold: bool,
    clk_div: u32,
    clk_src: u32,
    shift: u32,
}

impl MCITiming {
    pub fn new() -> Self {
        MCITiming {
            use_hold: false,
            clk_div: 0,
            clk_src: 0,
            shift: 0,
        }
    }
}

trait PadDelay {
    fn pad_delay(&self,id: u32);
}

impl PadDelay for MCITiming {
    fn pad_delay(&self, id: u32) {
        todo!()
    }
}


//* 核心数据结构 */
pub struct MCI {
    config: MCIConfig,
    reg: Reg,
    is_ready: bool,
    prev_cmd: u32,
    curr_timing: MCITiming,
}
impl MCI {
    pub fn new(reg_base: NonNull<u8>) -> Self {
        MCI {
            reg: Reg::new(reg_base),
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

    pub fn init_external_clk(&self) -> FsdifResult {
        let reg_val = uhs_reg(0, 0, 0x5) | FsdifClkSrc::UHS_EXT_CLK_ENA;
        if 0x502 == reg_val.bits() {
            debug!("invalid uhs config");
        }
        //? 这里可能需要抽象出一个update_external_clk的函数
        self.reg.write_reg(FsdifClkSrc::from_bits_truncate(0));
        self.uhs_reg_set(0,0,0x5);
        self.reg.modify_reg(|reg|{
            FsdifClkSrc::UHS_EXT_CLK_ENA | reg
        });
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
            uhs_clk_drv(drv_phase) | uhs_clk_samp(samp_phase) |uhs_clk_div(clk_div)}
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
    }

    pub fn clock_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifClkEn::CCLK_ENABLE | reg
            } else {
                !FsdifClkEn::CCLK_ENABLE & reg
            }
        });
    }

    pub fn clock_src_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifClkSrc::UHS_EXT_CLK_ENA |reg
            } else {
                !FsdifClkSrc::UHS_EXT_CLK_ENA & reg
            }
        });
        
    }

    pub fn voltage_1_8v_set(&self,enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifUhsReg::VOLT_180 | reg
            } else {
                !FsdifUhsReg::VOLT_180 & reg
            }
        });
    }

    pub fn bus_width_set(&self, width: u32) -> FsdifResult {
        let reg_val:FsdifCType;
        if width == 1 {
            reg_val = FsdifCType::CARD0_WIDTH1_8BIT;
        } else if width == 4 {
            reg_val = FsdifCType::CARD0_WIDTH2_4BIT;
        } else if width == 8 {
            reg_val = FsdifCType::CARD0_WIDTH1_8BIT;
        } else {
            return Err(FsdifError::NotSupport);
        }
        self.reg.write_reg(reg_val);
        Ok(())
    }

    pub fn ctrl_reset(&self, reset_bits: FsdifCtrl) -> FsdifResult {
        self.reg.write_reg(reset_bits.clone());
        self.reg.wait_for(|reg| {
            (reset_bits & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn send_private_cmd(&self, cmd:FsdifCmd, arg: u32) -> FsdifResult {
        self.reg.wait_for(|reg| {
            (FsdifStatus::DATA_BUSY & reg).bits() == 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        self.reg.write_reg(FsdifCmdArg::from_bits_truncate(arg));
        unsafe { dsb() };/* drain writebuffer */
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

    pub fn reset(&self) -> FsdifResult {
        /* set fifo */
        self.fifoth_set(
            FsdifFifoThDmaTransSize::DmaTrans8, 
            FSDIF_RX_WMARK, 
            FSDIF_TX_WMARK);
        debug!("fifoth set success");
        /* set card threshold */
        self.reg.write_reg( 
            FsdifCardThrctl::CARDRD |
            FsdifFifoDepth::Depth8.card_thrctl_threshold().into());
        debug!("card threshold set success");
        /* disable clock and update ext clk */
        self.clock_set(true);
        debug!("clock set success");
        /* set 1st clock */
        self.init_external_clk()?;
        debug!("external clock init success");
        /* power on */
        self.power_set(true);
        self.clock_set(false);
        self.clock_src_set(true);
        debug!("power on success");
        /* set voltage as 3.3v */
        self.voltage_1_8v_set(false);
        debug!("voltage set to 3.3v");
        /* set bus width as 1 */
        self.bus_width_set(1)?;
        debug!("bus width set to 1");
        /* reset controller and card */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET | FsdifCtrl::DMA_RESET)?;
        } else {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET)?;
        }
        debug!("controller reset success");
        /* send private command to update clock */
        self.send_private_cmd(FsdifCmd::UPD_CLK, 0)?;
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
        debug!("card reset success");
        /* clear interrupt status */
        self.reg.write_reg(FsdifInt::empty());
        let reg_val = self.reg.read_reg::<FsdifRawInts>();
        self.reg.write_reg(reg_val);
        self.reg.write_reg(FsdifDmacIntEn::empty());
        let reg_val = self.reg.read_reg::<FsdifDmacStatus>();
        self.reg.write_reg(reg_val);
        debug!("clear interrupt status success");
        /* enable card detect interrupt */
        if !self.config.non_removable {
            self.reg.modify_reg(|reg|{
                FsdifInt::CD_BIT | reg
            });
        }
        debug!("enable card detect interrupt success");
        /* enable controller and internal DMA */
        self.reg.modify_reg(|reg|{
            FsdifCtrl::INT_ENABLE | FsdifCtrl::USE_INTERNAL_DMAC | reg
        });
        debug!("enable controller and internal DMA success");
        /* reset descriptors and dma */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.descriptor_set(0);
            self.idma_reset();
        }
        debug!("reset descriptors and dma success");
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
        if self.is_ready{
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
                unsafe { dsb() };
                self.pio_write_data(cmd_data.data.buf)?;
            }
        }
        self.transfer_cmd(cmd_data)?;
        Ok(())
    }

    pub fn poll_wait_pio_end(self,cmd_data: &mut FSdifCmdData) -> FsdifResult{
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
            (FsdifRawInts::CMD_BIT & reg).contains(FsdifRawInts::CMD_BIT)
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        /* if need to read data, read fifo after send command */
        if read {
            info!("wait for PIO data to read ...");
            self.reg.wait_for(|reg|{
                (FsdifRawInts::DTO_BIT & reg).contains(FsdifRawInts::DTO_BIT)
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
struct FsdifBuf {
    buf: &'static mut [u32],
    buf_dma: u32,
    blksz: u32,
    blkcnt: u32,
}
pub struct FSdifCmdData {
    cmdidx: u32,
    cmdarg: u32,
    response: [u32; 4],
    flag: CmdFlag,
    data: FsdifBuf,
    success: bool
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
    ClkSpeed400KHz = 400_000,
    ClkSpeed25Mhz = 25_000_000,
    ClkSpeed26Mhz = 26_000_000, // mmc
    ClkSpeed50Mhz = 50_000_000,
    ClkSpeed52Mhz = 52_000_000, // mmc
    ClkSpeed66Mhz = 66_000_000, // mmc
    ClkSpeed100Mhz = 100_000_000,
}

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}