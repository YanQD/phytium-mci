use alloc::vec::Vec;

use super::MCIHost;

pub(crate) struct MCICardBase {
    pub host: Option<MCIHost>,
    pub is_host_ready: bool,
    pub no_interal_align: bool,
    pub internal_buffer: Vec<u8>,
    pub bus_clk_hz: u32,
    pub relative_address: u32,
    pub ocr: u32,
    pub block_size: u32,
}

impl MCICardBase {
    pub fn new() -> Self {
        MCICardBase {
            host: None,
            is_host_ready: false,
            no_interal_align: false,
            internal_buffer: Vec::new(),
            bus_clk_hz: 0,
            relative_address: 0,
            ocr: 0,
            block_size: 0,
        }
    }

    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        MCICardBase {
            host: None,
            is_host_ready: false,
            no_interal_align: false,
            internal_buffer: buffer,
            bus_clk_hz: 0,
            relative_address: 0,
            ocr: 0,
            block_size: 0,
        }
    }
    
}
