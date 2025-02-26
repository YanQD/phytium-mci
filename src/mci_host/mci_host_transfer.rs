use alloc::vec::Vec;

use super::constants::*;

pub(crate) struct MCIHostTransfer {
    data: Option<MCIHostData>,
    cmd: Option<MCIHostCmd>,
}

impl MCIHostTransfer {
    pub(crate) fn new() -> Self {
        MCIHostTransfer {
            data:None,
            cmd:None
        }
    }

    pub(crate) fn data(&self) -> Option<&MCIHostData> {
        self.data.as_ref()
    }

    pub(crate) fn set_data(&mut self,data:Option<MCIHostData>) {
        self.data = data
    }
    
    pub(crate) fn cmd(&self) -> Option<&MCIHostCmd> {
        self.cmd.as_ref()
    }

    pub(crate) fn set_cmd(&mut self,cmd:Option<MCIHostCmd>) {
        self.cmd = cmd
    }

    pub(crate) fn data_mut(&mut self) -> Option<&mut MCIHostData> {
        self.data.as_mut()
    }

    pub(crate) fn cmd_mut(&mut self) -> Option<&mut MCIHostCmd> {
        self.cmd.as_mut()
    }

}

pub(crate) struct MCIHostData {
    stream_transfer: bool,          // 指示是否为流数据传输命令
    enable_auto_command12: bool,    // 启用自动 CMD12
    enable_auto_command23: bool,    // 启用自动 CMD23
    enable_ignore_error: bool,      // 启用忽略错误以读取/写入所有数据
    data_type: u8,                  // 用于区分普通/调谐/启动数据
    block_size: usize,              // 块大小
    block_count: u32,               // 块数量
    rx_data: Option<Vec<u32>>,      // 用于保存读取数据的缓冲区
    tx_data: Option<Vec<u32>>,      // 用于写入数据的缓冲区
}

impl MCIHostData {
    pub(crate) fn stream_transfer(&self) -> bool {
        self.stream_transfer
    }

    pub(crate) fn enable_auto_command12(&self) -> bool {
        self.enable_auto_command12
    }

    pub(crate) fn enable_auto_command23(&self) -> bool {
        self.enable_auto_command23
    }

    pub(crate) fn enable_ignore_error(&self) -> bool {
        self.enable_ignore_error
    }

    pub(crate) fn data_type(&self) -> u8 {
        self.data_type
    }

    pub(crate) fn block_size(&self) -> usize {
        self.block_size
    }

    pub(crate) fn block_size_set(&mut self, block_size:usize) {
        self.block_size = block_size;
    } 

    pub(crate) fn block_count(&self) -> u32 {
        self.block_count
    }

    pub(crate) fn rx_data(&self) -> Option<&Vec<u32>> {
        self.rx_data.as_ref()
    }

    pub(crate) fn rx_data_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.rx_data.as_mut()
    }

    pub(crate) fn tx_data(&self) -> Option<&Vec<u32>> {
        self.tx_data.as_ref()
    }

    pub(crate) fn tx_data_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.tx_data.as_mut()
    }

}

pub(crate) struct MCIHostCmd {
    index: u32,                          // 命令索引
    argument: u32,                       // 命令参数
    cmd_type: MCIHostCmdType,            // 命令类型
    response_type: MCIHostResponseType,  // 命令响应类型
    response: [u32; 4],                  // 命令响应数据
    response_error_flags: u32,           // 响应错误标志
    flags: u32,                          // 命令标志
}

impl MCIHostCmd {

    pub(crate) fn new() -> Self {
        MCIHostCmd {
            index:0,
            argument:0,
            cmd_type:MCIHostCmdType::Normal,
            response_type:MCIHostResponseType::None,
            response:[0;4],
            response_error_flags:0,
            flags:0,
        }
    } 

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn set_index(&mut self,index:u32) {
        self.index = index
    }

    pub(crate) fn argument(&self) -> u32 {
        self.argument
    }

    pub(crate) fn set_argument(&mut self,argument:u32) {
        self.argument = argument
    }

    pub(crate) fn cmd_type(&self) -> MCIHostCmdType {
        self.cmd_type
    }

    pub(crate) fn response_type(&self) -> MCIHostResponseType {
        self.response_type
    }

    pub(crate) fn set_response_type(&mut self,response_type:MCIHostResponseType) {
        self.response_type = response_type
    }

    pub(crate) fn response(&self) -> &[u32; 4] {
        &self.response
    }

    pub(crate) fn response_mut(&mut self) -> &mut [u32;4] {
        &mut self.response
    }

    pub(crate) fn response_error_flags(&self) -> u32 {
        self.response_error_flags
    }

    pub(crate) fn flags(&self) -> u32 {
        self.flags
    }
}