use super::mci_data::MCIData;
use super::err::*;
use super::constants::*;
use super::regs::*;

#[derive(Debug)]
pub struct MCICmdData<'a> {
    cmdidx: u32,
    cmdarg: u32,
    pub(crate) response: [u32; 4],
    flag: MCICmdFlag,
    data: Option<MCIData<'a>>,
    pub(crate) success: bool
}

impl<'a> MCICmdData<'a> {
    pub(crate) fn new() -> Self {
        MCICmdData {
            cmdidx: 0,
            cmdarg: 0,
            response: [0; 4],
            flag: MCICmdFlag::empty(),
            data: None,
            success: false
        }
    }
    pub(crate) fn cmdidx(&self) -> u32 {
        self.cmdidx
    }
    pub(crate) fn cmdarg(&self) -> u32 {
        self.cmdarg
    }
    pub(crate) fn flag(&self) -> MCICmdFlag {
        self.flag
    }
    pub(crate) fn get_data(&mut self) -> Option<&mut MCIData<'a>> {
        if self.data.is_some() {
            Some(self.data.as_mut().unwrap())
        } else {
            None
        }
    }
}