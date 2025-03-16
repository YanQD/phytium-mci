#[allow(unused)]

mod constants;
mod mci_host_config;
mod mci_host_device;
mod mci_host_card;
mod mci_host_transfer;
pub mod mci_sdif;
mod err;
mod sd;
mod mci_card_base;
mod mci_host_card_detect;

use core::{cell::RefCell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc};

use constants::*;
use err::{MCIHostError, MCIHostStatus};
use mci_host_card_detect::MCIHostCardDetect;
use mci_host_config::MCIHostConfig;
use mci_host_device::MCIHostDevice;
use mci_host_card::MCIHostCard;
use mci_host_transfer::{MCIHostCmd, MCIHostTransfer};

type MCIHostCardIntFn = fn();

#[allow(unused)]
pub struct MCIHost {
    dev: Box<dyn MCIHostDevice>,
    config: MCIHostConfig,                  
    curr_voltage: MCIHostOperationVoltage,  
    curr_bus_width: u32,                    
    curr_clock_freq: u32,                   

    source_clock_hz: u32,                   
    capability: MCIHostCapability,          
    max_block_count: u32,                   
    max_block_size: u32,                    
    tuning_type: u8,                        

    card: Option<Rc<RefCell<Box<dyn MCIHostCard>>>>,
    cd: Option<Rc<RefCell<MCIHostCardDetect>>>,         // 卡检测
    card_int: MCIHostCardIntFn,

    //? 这里 uint8_t tuningType sdmmc_osa_event_t hostEvent sdmmc_osa_mutex_t lock 都没有移植
}

#[allow(unused)]
impl MCIHost {

    pub(crate) fn new(dev: Box<dyn MCIHostDevice>, config: MCIHostConfig) -> Self {
        let capability = MCIHostCapability::empty();
        let max_block_count = 0;
        let max_block_size = 0;
        let tuning_type = 0;
        let card = None;
        let cd = None;
        let card_int = || {};

        MCIHost {
            dev,
            config,
            curr_voltage: MCIHostOperationVoltage::None,
            curr_bus_width: 0,
            curr_clock_freq: 0,
            source_clock_hz: 0,
            capability,
            max_block_count,
            max_block_size,
            tuning_type,
            card,
            cd,
            card_int,
        }
    }

    pub(crate) fn card_select(&mut self,relative_address:u32,is_selected:bool) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();

        command.set_index(MCIHostCommonCmd::SelectCard as u32);
        if is_selected {
            command.set_argument(relative_address << 16);
            command.set_response_type(MCIHostResponseType::R1);
        } else {
            command.set_argument(0);
            command.set_response_type(MCIHostResponseType::None);
        }

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        let err = self.dev.transfer_function(&mut content);
        
        let command = content.cmd().unwrap();
        let response = command.response();

        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0{
            return Err(MCIHostError::TransferFailed);
        }

        Ok(())
    }

    pub(crate) fn application_command_send(&mut self, relative_address: u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.set_index(MCIHostCommonCmd::ApplicationCommand as u32);
        command.set_argument(relative_address << 16);
        command.set_response_type(MCIHostResponseType::R1);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
    
        let err = self.dev.transfer_function(&mut content);
        
        let command = content.cmd().unwrap();
        let response = command.response();
    
        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0 {
            return Err(MCIHostError::TransferFailed);
        }
    
        if response[0] & MCIHostCardStatusFlag::APPLICATION_COMMAND.bits() == 0 {
            return Err(MCIHostError::CardNotSupport);
        }
    
        Ok(())
    }
 
    pub(crate) fn block_count_set(&mut self,block_count:u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();

        command.set_index(MCIHostCommonCmd::SetBlockCount as u32);
        command.set_argument(block_count);
        command.set_response_type(MCIHostResponseType::R1);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content);

        let command = content.cmd().unwrap();
        let response = command.response();

        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0{
            return Err(MCIHostError::TransferFailed);
        }

        Ok(())
    }

    pub(crate) fn go_idle(&mut self) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.set_index(MCIHostCommonCmd::GoIdleState as u32);
        
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content);
        
        if err.is_err() {
            return Err(MCIHostError::TransferFailed);
        }
        
        Ok(())
    }

    pub(crate) fn block_size_set(&mut self, block_size: u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.set_index(MCIHostCommonCmd::SetBlockLength as u32);
        command.set_argument(block_size);
        command.set_response_type(MCIHostResponseType::R1);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content);
        
        let command = content.cmd().unwrap();
        let response = command.response();
    
        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0 {
            return Err(MCIHostError::TransferFailed);
        }
    
        Ok(())
    }
    
    pub(crate) fn card_inactive_set(&mut self) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.set_index(MCIHostCommonCmd::GoInactiveState as u32);
        command.set_argument(0);
        command.set_response_type(MCIHostResponseType::None);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content);
        
        if err.is_err() {
            return Err(MCIHostError::TransferFailed);
        }
    
        Ok(())
    }

    pub(crate) fn init(&mut self, addr: NonNull<u8>) -> MCIHostStatus {
        self.dev.init(addr)
    }

}

#[allow(unused)]
impl MCIHost {
    pub(crate) fn cd(&self) -> Option<Rc<RefCell<MCIHostCardDetect>>> {
        self.cd.as_ref().map(|cd| Rc::clone(cd))
    }

    pub(crate) fn max_block_count(&self) -> u32 {
        self.max_block_count
    }
    
    pub(crate) fn max_block_count_set(&mut self, max_block_count: u32) {
        self.max_block_count = max_block_count;
    }

    pub(crate) fn max_block_size(&self) -> u32 {
        self.max_block_size
    }

    pub(crate) fn max_block_size_set(&mut self, max_block_size: u32) {
        self.max_block_size = max_block_size;
    }

    pub(crate) fn capability(&self) -> MCIHostCapability {
        self.capability
    }

    pub(crate) fn capability_set(&mut self, capability: MCIHostCapability) {
        self.capability = capability;
    }
}


