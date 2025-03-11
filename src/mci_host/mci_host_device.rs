use core::ptr::NonNull;

use alloc::vec::Vec;

use crate::MCICmdData;

use super::constants::*;
use super::err::*;
use super::mci_host_card_detect::MCIHostCardDetect;
use super::mci_host_transfer::MCIHostTransfer;
use super::MCIHostCardIntFn;
use super::mci_sdif::constants::SDStatus;

pub(crate) trait MCIHostDevice {

    fn init(&mut self, addr: NonNull<u8>)-> MCIHostStatus;
    fn do_init(&mut self, addr: NonNull<u8>) -> MCIHostStatus;
    /* sdmmc host operations */
    fn deinit(&mut self);
    fn reset(&self) -> MCIHostStatus;

    /* set sdmmc host mode and get host status */
    fn switch_to_voltage(&mut self, voltage: MCIHostOperationVoltage) -> MCIHostStatus;
    fn execute_tuning(&self, tuning_cmd: u32, rev_buf: &mut Vec<u32>, block_size: u32) -> MCIHostStatus;  
    fn enable_ddr_mode(&self, enable: bool, nibble_pos: u32);
    fn enable_hs400_mode(&self, enable: bool);
    fn enable_strobe_dll(&self, enable: bool);
    fn get_signal_line_status(&self, signal_line: u32) -> bool;
    fn convert_data_to_little_endian(&self, data: &mut Vec<u32>, word_size: usize, format: MCIHostDataPacketFormat) -> MCIHostStatus;

    /* card related functions */
    // todo 这里引入的 MCIHostCardDetect 如果是实现 MCIHostDevice 的一定有的成员,这里留一个获取它的接口 get_MCIHostDevice
    fn card_detect_init(&self, cd: &MCIHostCardDetect) -> MCIHostStatus;
    fn card_power_set(&self, enable: bool);
    fn force_clock_on(&self, enable: bool);
    fn card_int_enable(&self, enable: bool) -> MCIHostStatus;
    // todo 同上
    fn card_int_init(&self, sdio_int: &MCIHostCardIntFn) -> MCIHostStatus;
    fn card_bus_width_set(&self, data_bus_width: MCIHostBusWdith);
    fn card_detect_status_polling(&self, wait_card_status: SDStatus, timeout: u32) -> MCIHostStatus;
    fn card_detect_status(&self) -> SDStatus;
    fn card_active_send(&self);
    fn card_clock_set(&mut self, target_clock: u32) -> u32;
    fn card_is_busy(&self) -> bool;

    /* data transfer related functions */
    fn pre_command(&mut self,content: &mut MCIHostTransfer) -> MCIHostStatus;
    fn covert_command_info(&self, in_trans: &mut MCIHostTransfer) -> MCICmdData;
    fn transfer_function(&mut self, content: &mut MCIHostTransfer) -> MCIHostStatus;

    /* boot related functions */
    // todo 永远不会用到它们
    // fn start_boot(&self, host_config: &MCIHostBootConfig, cmd: &MCIHostCmd, buffer: &mut [u8]) -> MCIHostStatus;
    // fn read_boot_data(&self, host_config: &MCIHostBootConfig, buffer: &mut [u8]) -> MCIHostStatus;
    // fn enable_boot(&self, enable: bool);

}
