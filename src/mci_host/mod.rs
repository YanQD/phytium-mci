#[allow(unused)]

mod constants;
mod mci_host_config;
mod mci_host_device;
mod mci_host_card;
mod mci_host_card_detect;
mod mci_host_transfer;
mod mci_sdif;
mod err;

use alloc::boxed::Box;

use constants::*;
use err::{MCIHostError, MCIHostStatus};
use mci_host_config::MCIHostConfig;
use mci_host_device::MCIHostDevice;
use mci_host_card::MCIHostCard;
use mci_host_card_detect::MCIHostCardDetect;
use mci_host_transfer::{MCIHostCmd, MCIHostTransfer};

type MCIHostCardInt = fn();

pub struct MCIHost {
    dev: Box<dyn MCIHostDevice>,
    config: MCIHostConfig,                  // 主机控制器配置
    curr_voltage: MCIHostVoltage,           // 当前电压
    curr_bus_width: u32,                    // 当前总线宽度
    curr_clock_freq: u32,                   // 当前时钟频率

    source_clock_hz: u32,                   // 主机配置：源时钟频率
    capability: u32,                        // 主机控制器能力
    max_block_count: u32,                   // 主机控制器最大块计数
    max_block_size: u32,                    // 主机控制器最大块大小
    tuning_type: u8,                        // 主机调优类型

    card: Box<dyn MCIHostCard>,
    cd: MCIHostCardDetect,
    card_int: MCIHostCardInt,

    //? 这里 uint8_t tuningType sdmmc_osa_event_t hostEvent sdmmc_osa_mutex_t lock 都没有移植
}

impl MCIHost {
    pub(crate) fn cd(&self) -> &MCIHostCardDetect {
        &self.cd
    }

    pub(crate) fn card_select(&mut self,relative_address:u32,is_selected:bool) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();

        command.set_index(MCIHostCommonCommand::SelectCard as u32);
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
    
        command.set_index(MCIHostCommonCommand::ApplicationCommand as u32);
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

        command.set_index(MCIHostCommonCommand::SetBlockCount as u32);
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
    
        command.set_index(MCIHostCommonCommand::GoIdleState as u32);
        
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
    
        command.set_index(MCIHostCommonCommand::SetBlockLength as u32);
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
    
        command.set_index(MCIHostCommonCommand::GoInactiveState as u32);
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

}


