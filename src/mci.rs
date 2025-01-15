#![allow(unused)] 
#![feature(asm)]
use core::arch::asm;
use core::ptr::NonNull;
use core::time::Duration;
use crate::constants::*;
use crate::regs::*;
use crate::err::{FsdifError, FsdifResult};

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

pub struct MCI {
    reg: Reg,
    config: MCIConfig,
}
impl MCI {
    pub fn new(reg_base: NonNull<u8>) -> Self {
        MCI {
            reg: Reg::new(reg_base),
            config: MCIConfig::new(),
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
            (reset_bits & reg).bits() != 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn send_private_cmd(&self, cmd:FsdifCmd, arg: u32) -> FsdifResult {
        self.reg.wait_for(|reg| {
            (FsdifStatus::DATA_BUSY & reg).bits() != 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        dsb(); /* drain writebuffer */
        self.reg.write_reg(FsdifCmd::START | cmd);
        self.reg.wait_for(|reg|{
            (FsdifCmd::START & reg).bits() != 0
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

    pub fn reset(&self) -> FsdifResult {
        /* set fifo */
        self.fifoth_set(
            FsdifFifoThDmaTransSize::DmaTrans8, 
            FSDIF_RX_WMARK, 
            FSDIF_TX_WMARK);
        /* set card threshold */
        self.reg.write_reg( 
            FsdifCardThrctl::CARDRD |
            FsdifFifoDepth::Depth8.card_thrctl_threshold().into());
        /* disable clock and update ext clk */
        self.clock_set(true);
        /* set 1st clock */
        self.init_external_clk()?;
        /* power on */
        self.power_set(true);
        self.clock_set(false);
        self.clock_src_set(true);
        /* set voltage as 3.3v */
        self.voltage_1_8v_set(false);
        /* set bus width as 1 */
        self.bus_width_set(1)?;
        /* reset controller and card */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET | FsdifCtrl::DMA_RESET)?;
        } else {
            self.ctrl_reset(FsdifCtrl::FIFO_RESET)?;
        }
        /* send private command to update clock */
        self.send_private_cmd(FsdifCmd::UPD_CLK, 0)?;
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
        /* clear interrupt status */
        self.reg.write_reg(FsdifInt::empty());
        let reg_val = self.reg.read_reg::<FsdifRawInts>();
        self.reg.write_reg(reg_val);
        
        self.reg.write_reg(FsdifDmacIntEn::empty());
        let reg_val = self.reg.read_reg::<FsdifDmacStatus>();
        self.reg.write_reg(reg_val);
        /* enable card detect interrupt */
        if !self.config.non_removable {
            self.reg.modify_reg(|reg|{
                FsdifInt::CD_BIT | reg
            });
        }
        /* enable controller and internal DMA */
        self.reg.modify_reg(|reg|{
            FsdifCtrl::INT_ENABLE | FsdifCtrl::USE_INTERNAL_DMAC | reg
        });
        /* reset descriptors and dma */
        if self.config.trans_mode == FsDifTransMode::DmaTransMode {
            self.descriptor_set(0);
            self.idma_reset();
        }
        Ok(())
    }

    pub fn init_external_clk(&self) -> FsdifResult {
        let reg_val = uhs_reg(0, 0, 0x5) | FsdifClkSrc::UHS_EXT_CLK_ENA;
        if 0x502 == reg_val.bits() {
            return Err(FsdifError::NotSupport);
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

}


// 定义传输模式枚举
#[derive(Debug, PartialEq)]
pub enum FsDifTransMode {
    DmaTransMode,      // DMA传输模式
    PioTransMode,      // PIO传输模式（通过读/写Fifo）
}

// 定义中断类型枚举
#[derive(Debug, PartialEq)]
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
fn dsb() {
    unsafe {
        asm!("dsb",":",":",":","memory");
    }
}
