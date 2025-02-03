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

use crate::sleep;

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
            if target_timing.pad_delay as usize != fsdif_sdifdelay_null as usize {
                (target_timing.pad_delay)(self.config.instance_id);
            }
            /* update clock source setting */
            self.update_exteral_clk(FsdifClkSrc::from_bits_retain(target_timing.clk_src))?;
            self.clock_set(false);
            /* update clock for clock source */
            if cur_cmd_index == FsDifSDIndivCommand::VoltageSwitch as u32 {
                self.send_private_cmd11(reg_val | cmd_reg)?;
            } else {
                self.send_private_cmd(reg_val, 0)?;
            }
            /* set clock divider */
            self.reg.write_reg(FsdifClkDiv::from_bits_truncate(target_timing.clk_div));
            self.reg.write_reg(FsdifEnableShift::from_bits_truncate(target_timing.shift));
            info!("clk_src: 0x{:x} clk_div: 0x{:x}, shift: 0x{:x}",
                self.reg.read_reg::<FsdifClkSrc>(),
                self.reg.read_reg::<FsdifClkDiv>(),
                self.reg.read_reg::<FsdifEnableShift>());
            self.clock_set(true);
            /* update clock for clock divider */
            if cur_cmd_index == FsDifSDIndivCommand::VoltageSwitch as u32 {
                self.send_private_cmd11(reg_val | cmd_reg)?;
            } else {
                self.send_private_cmd(reg_val, 0)?;
            }
        } else {
            /* close bus clock in case target clock is 0 */
            self.clock_set(false);

            if cur_cmd_index == FsDifSDIndivCommand::VoltageSwitch as u32 {
                self.send_private_cmd11(reg_val | cmd_reg)?;
            } else {
                self.send_private_cmd(reg_val, 0)?;
            }
        }
        Ok(())
    }

    pub fn select_bus_timing(&self) -> FsdifResult {
        /* group 1, function 1 ->high speed mode*/
        
        /* card is in UHS_I mode */ // todo 暂时不需要
        Ok(())
    }

    pub fn check_if_card_busy(&self) -> bool {
        let reg_val = self.reg.read_reg::<FsdifStatus>();
        (FsdifStatus::DATA_BUSY & reg_val).bits() != 0
    }

    pub fn prob_bus_voltage(&self) -> FsdifResult {
        let mut application_command41_argument = SdOcrFlag::empty(); /* OCR arguments */
        /* 3.3V voltage should be supported as default */
        application_command41_argument |= SdOcrFlag::VDD_29_30 | SdOcrFlag::VDD_32_33 | SdOcrFlag::VDD_34_35;
        /*
        * If card is high capacity (SDXC or SDHC), and supports 1.8V signaling,
        * switch to new signal voltage using "signal voltage switch procedure"
        * described in SD specification
        */
        // todo 这里高速的部分用不到
        // todo card->operationVoltage = kSDMMC_OperationVoltage330V;
        /* send card active */ //* 空函数 */

        loop {
            /* card go idle */
            self.go_idle()?; // CMD0
            match self.send_interface_condition() { /* CMD8 */
                Ok(_) => {
                    /* SDHC or SDXC card */
                    application_command41_argument |= SdOcrFlag::CARD_CAPACITY_SUPPORT;
                    // todo card->flags |= (uint32_t)kSD_SupportSdhcFlag;
                },
                Err(_) => {
                    /* SDSC card */
                    self.go_idle()?; /* make up for legacy card which do not support CMD8 */
                }
            }
            self.application_send_opration_condition(application_command41_argument)?; /* ACMD41 */
            /* Set card interface condition according to SDHC capability and card's supported interface condition. */
            /* check if card support 1.8V */ //* 这里可以暂时跳过 */
            break;
        }

        Ok(())
    }

    pub fn card_power_set(&self, enable: bool) {
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifPwrEn::ENABLE | reg
            } else {
                !FsdifPwrEn::ENABLE & reg
            }
        });
    }

    //* 因为暂时不会被调用而不去实现 */
    pub fn io_voltage_switch(&self) {
        todo!()
    }

    pub fn switch_to_voltage(&self,voltage:SdmmcOperationVoltage)  {
        match voltage {
            SdmmcOperationVoltage::V330V => {
                self.voltage_1_8v_set(false);
            }
            SdmmcOperationVoltage::V300V => {
                self.voltage_1_8v_set(false);
            }
            SdmmcOperationVoltage::V180V => {
                self.voltage_1_8v_set(true);
            }
            _ => {
                error!("Invalid voltage value !!!");
            }
        }
    }

    pub fn update_exteral_clk(&self,uhs_reg:FsdifClkSrc) -> FsdifResult {
        self.reg.write_reg(FsdifClkSrc::from_bits_truncate(0));
        self.reg.write_reg(uhs_reg);
        self.reg.wait_for(|reg| {
            (FsdifClkSts::READY & reg).bits() != 0
        }, Duration::from_millis((FSDIF_TIMEOUT / 100).into()), Some(100))?;
        Ok(())
    }

    pub fn init_external_clk(&self) -> FsdifResult {
        let reg_val = 
        FsdifClkSrc::uhs_reg(0, 0, 0x5) | 
        FsdifClkSrc::UHS_EXT_CLK_ENA;
        if 0x502 == reg_val.bits() {
            info!("invalid uhs config"); 
        }
        self.update_exteral_clk(reg_val)?;
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
        info!("clock set to 0x{:x}",self.reg.read_reg::<FsdifClkEn>()); 
    }

    pub fn clock_src_set(&self, enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifClkSrc::UHS_EXT_CLK_ENA |reg
            } else {
                !FsdifClkSrc::UHS_EXT_CLK_ENA & reg
            }
        });
        info!("clock src set to 0x{:x}",self.reg.read_reg::<FsdifClkSrc>());  
    }

    pub fn voltage_1_8v_set(&self,enable:bool){
        self.reg.modify_reg(|reg| {
            if enable {
                FsdifUhsReg::VOLT_180 | reg
            } else {
                !FsdifUhsReg::VOLT_180 & reg
            }
        });
        info!("voltage set to 0x{:x}",self.reg.read_reg::<FsdifUhsReg>());  
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

    //** 专门给CMD11 也就是 Switch Voltage 设计的 */
    pub fn send_private_cmd11(&self,cmd:FsdifCmd) -> FsdifResult {
        unsafe { dsb() };/* drain writebuffer */
        self.reg.write_reg(FsdifCmd::START | cmd);
        self.reg.wait_for(|reg|{
            (FsdifCmd::START & reg).bits() == 0
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
            FSDIF_TX_WMARK);  
        debug!("fifoth set success");
        /* set card threshold */
        self.reg.write_reg( 
            FsdifCardThrctl::CARDRD |
            FsdifFifoDepth::Depth8.card_thrctl_threshold().into());
        info!("card threshold set to 0x{:x}",
        (FsdifCardThrctl::CARDRD |
        FsdifFifoDepth::Depth8.card_thrctl_threshold().into()).bits());  
        debug!("card threshold set success");
        /* disable clock and update ext clk */
        self.clock_set(false);  
        debug!("clock set success");
        /* set 1st clock */
        self.init_external_clk()?;  
        debug!("external clock init success");
        /* power on */
        self.power_set(true);  
        self.clock_set(true);  
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
        /* clear interrupt status */  
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
        info!("enable controller and internal DMA to 0x{:x}",self.reg.read_reg::<FsdifCtrl>());  
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

    //* CMD0 */
    pub fn go_idle(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::GoIdleState as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::empty(),
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

    //* CMD2 */
    pub fn all_send_cid(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::AllSendCid as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::EXP_LONG_RESP | CmdFlag::NEED_RESP_CRC,
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
        // todo DecodeCid
        Ok(())
    }

    //* CMD3 */
    pub fn send_rca(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifSDIndivCommand::SendRelativeAddress as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
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
        // todo card->relativeAddress = (command.response[0U] >> 16U);
        Ok(())
    }

    //* CMD7 */
    pub fn select_card(&self,relative_address:u32,is_selected:bool) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SelectCard as u32,
            cmdarg: if is_selected { relative_address << 16 } else { 0 },
            response: [0; 4],
            flag: if is_selected{CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC} else {CmdFlag::empty()},
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
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::SDMMC_R1_ALL_ERROR_FLAG.bits() != 0 {
            return Err(FsdifError::InvalidState);
        }
        Ok(())
    }

    //* CMD8 */
    pub fn send_interface_condition(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifSDIndivCommand::SendInterfaceCondition as u32,
            cmdarg: 0x1AA,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
            data: FsdifBuf {
                buf: &mut [],
                buf_dma: 0,
                blksz: 0,
                blkcnt: 0,
            },
            success: false,
        };
        let i = 0;
        while i < FSL_SDMMC_MAX_CMD_RETRIES {

            match self.pio_transfer(&cmd_data) {
                Ok(_) => break,
                _ => continue,
            }
            match self.poll_wait_pio_end(&mut cmd_data) {
                Ok(_) => break,
                _ => continue,
                
            }
            if cmd_data.response[0] & 0xFF != 0xAA {
                return Err(FsdifError::NotSupport);
            }
            i = i + 1;
        }
        if i >= FSL_SDMMC_MAX_CMD_RETRIES {
            return Err(FsdifError::CmdTimeout);
        }
        Ok(())
    }

    //* CMD9 */
    pub fn send_csd(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SendCsd as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::EXP_LONG_RESP | CmdFlag::NEED_RESP_CRC,
            data: FsdifBuf {
                buf: &mut [],
                buf_dma: 0,
                blksz: 0,
                blkcnt: 0,
            },
            success: false,
        };
        /* The response is from bit 127:8 in R2, corrisponding to command.response[3U]:command.response[0U][31U:8]. */
        // todo SD_DecodeCsd(card, (uint32_t *)(uintptr_t)card->internalBuffer);
        Ok(())
    }

    //* CMD11 */
    pub fn voltage_switch(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifSDIndivCommand::VoltageSwitch as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
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
        /*
        * Card should drive CMD and DAT[3:0] signals low at the next clock
        * cycle. Some cards will only drive these
        * lines low briefly, so we should check as soon as possible
        */
        if !self.check_if_card_busy() {
            /* Delay 1ms to allow card to drive lines low */
            sleep(Duration::from_millis(1));
            if !self.check_if_card_busy() {
                /* Card did not drive CMD and DAT lines low */
                info!("Card did not drive DAT lines low");
                return Err(FsdifError::Busy);
            }
        }

        /*
        * Per SD spec (section "Timing to Switch Signal Voltage"),
        * host must gate clock at least 5ms.
        */
        // todo 暂时好像不需要switch
        // self.clk_freq_set(0);
        // /* switch io voltage */
        // self.io_voltage_switch(); 

        // /* Gate for 10ms, even though spec requires 5 */
        // sleep(Duration::from_millis(10));

        // /* Restart the clock */
        // self.clk_freq_set(self.config.bus_clk_hz)?;

        /*
        * If SD does not drive at least one of
        * DAT[3:0] high within 1ms, switch failed
        */
        sleep(Duration::from_millis(1));
        if self.check_if_card_busy() {
            info!("Card failed to switch voltages");
            return Err(FsdifError::Busy);
        }
        info!("Card switched to 1.8V signaling");
        Ok(())
    }

    //* CMD16 */
    pub fn block_size_set(&self, blksize: u32) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SetBlockLength as u32,
            cmdarg: blksize,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
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
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::SDMMC_R1_ALL_ERROR_FLAG.bits() != 0 {
            return Err(FsdifError::InvalidState);
        }
        Ok(())
    }

    //* CMD23 */
    pub fn block_count_set(&self, blkcnt: u32) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::SetBlockCount as u32,
            cmdarg: blkcnt,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
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
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::SDMMC_R1_ALL_ERROR_FLAG.bits() != 0 {
            return Err(FsdifError::InvalidState);
        }
        Ok(())
    }

    //* CMD55 */
    pub fn send_application_command(&self, relative_address:u32) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: FsDifCommand::ApplicationCommand as u32,
            cmdarg: relative_address << 16,
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
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
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::SDMMC_R1_ALL_ERROR_FLAG.bits() != 0 {
            return Err(FsdifError::InvalidState);
        }
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::APPLICATION_COMMAND.bits() == 0 {
            return Err(FsdifError::NotSupport);
        }
        Ok(())
    }
}

//* ACMD */
impl MCI {

    //* ACMD6 */
    pub fn data_bus_width(&self,width:SdmmcBusWidth) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: SdApplicationCommand::SetBusWidth as u32,
            cmdarg: match width {
                SdmmcBusWidth::BusWidth1Bit => 0,
                SdmmcBusWidth::BusWidth4Bit => 2,
                default => {
                    return Err(FsdifError::NotSupport);
                }
            }, 
            response: [0; 4],
            flag: CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
            data: FsdifBuf {
                buf: &mut [],
                buf_dma: 0,
                blksz: 0,
                blkcnt: 0,
            },
            success: false,
        };
        self.send_application_command(0)?;
        if cmd_data.response[0] & SdmmcR1CardStatusFlag::SDMMC_R1_ALL_ERROR_FLAG.bits() != 0 {
            return Err(FsdifError::InvalidState);
        }
        Ok(())
    }
    //* ACMD41 */
    pub fn application_send_opration_condition(&self,arg: SdOcrFlag) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: SdApplicationCommand::SendOperationCondition as u32,
            cmdarg: arg.bits(),
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
        let i = 0;
        while i < FSL_SDMMC_MAX_CMD_RETRIES {
            if let Ok(_) = self.send_application_command(0){
                match self.pio_transfer(&cmd_data) {
                    Ok(_) => break,
                    _ => continue,
                }
                match self.poll_wait_pio_end(&mut cmd_data) {
                    Ok(_) => break,
                    _ => continue,
                }
                /* Wait until card exit busy state. */
                if cmd_data.response[0] & SdOcrFlag::POWER_UP_BUSY.bits() != 0 {
                    /* high capacity check */
                    if  cmd_data.response[0] & SdOcrFlag::CARD_CAPACITY_SUPPORT.bits() !=0 {
                        // todo card->flags |= (uint32_t)kSD_SupportHighCapacityFlag;
                        info!("Is high capcity card > 2GB");
                    }
                    /* 1.8V support */
                    if cmd_data.response[0] & SdOcrFlag::SWITCH_18_ACCEPT.bits() != 0 {
                        // todo card->flags |= (uint32_t)kSD_SupportVoltage180v;
                        info!("Is UHS card support 1.8v");
                    }else {
                        info!("Not UHS card only support 3.3v")
                    }
                    // todo card->ocr = command.response[0U];
                }
            } else {
                continue;
            }
            i = i + 1;
            sleep(Duration::from_millis(10));
        }
        if i >= FSL_SDMMC_MAX_CMD_RETRIES {
            return Err(FsdifError::CmdTimeout);
        }
        Ok(())
    }

    //* ACMD51 */
    pub fn send_scr(&self) -> FsdifResult {
        let mut cmd_data = FSdifCmdData {
            cmdidx: SdApplicationCommand::SendScr as u32,
            cmdarg: 0,
            response: [0; 4],
            flag: CmdFlag::READ_DATA | CmdFlag::EXP_RESP | CmdFlag::NEED_RESP_CRC,
            data: FsdifBuf {
                buf: &mut [0;8],
                buf_dma: 0,
                blksz: 8,
                blkcnt: 1,
            },
            success: false,
        };
        let relative_address = 0;
        self.send_application_command(relative_address)?;
        self.pio_transfer(&cmd_data)?;
        self.poll_wait_pio_end(&mut cmd_data)?;
         /* according to spec. there are two types of Data packet format for SD card
            1. Usual data (8-bit width), are sent in LSB first
            2. Wide width data (SD Memory register), are shifted from the MSB bit, 
        e.g. ACMD13 (SD Status), ACMD51 (SCR) */
        // todo SDMMCHOST_ConvertDataToLittleEndian(card->host, rawScr, 2U, kSDMMC_DataPacketFormatMSBFirst);
        /* decode scr */
        // todo SD_DecodeScr(card, rawScr);
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
        // self.interrupt_mask_set(FsDifIntrType::GeneralIntr, FsdifInt::INTS_DATA_MASK.bits(), true);
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
                unsafe { dsb() };
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
        self.get_cmd_response(cmd_data)?;
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

