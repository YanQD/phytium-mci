use alloc::vec::Vec;

use super::MCIHost;

pub(crate) struct MCICardBase {
    pub host: Option<MCIHost>,
    pub is_host_ready: bool,
    pub bo_interal_align: bool,
    pub internal_buffer: Vec<u8>,
    pub bus_clk_hz: u32,
    pub relative_address: u32,
    pub ocr: u32,
    pub block_size: u32,
}
