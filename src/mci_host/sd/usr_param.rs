use crate::mci_host::mci_host_card_detect::MCIHostCardDetect;
use super::{constants::SdTimingMode, io_voltage::SdIoVoltage};

pub(crate) struct SdUsrParam {
    sd_pwr: Option<SdPwrFn>,
    power_on_delay_ms: u32,
    power_off_delay_ms: u32,
    io_strength: Option<SdIoStrengthFn>,
    io_voltage: Option<SdIoVoltage>,
    cd: Option<MCIHostCardDetect>,
    max_freq: u32,
    capbility: u32,
}

type SdPwrFn = fn(bool);
type SdIoStrengthFn = fn(SdTimingMode);

impl SdUsrParam {
    pub fn sd_pwr(&self) -> Option<&SdPwrFn> {
        self.sd_pwr.as_ref()
    }

    pub fn io_strength(&self) -> Option<&SdIoStrengthFn> {
        self.io_strength.as_ref()
    }

    pub fn power_on_delay_ms(&self) -> u32 {
        self.power_on_delay_ms
    }
    
    pub fn power_off_delay_ms(&self) -> u32 {
        self.power_off_delay_ms
    }

    pub fn io_voltage(&self) -> Option<&SdIoVoltage> {
        self.io_voltage.as_ref()
    }

    pub fn cd(&self) -> Option<&MCIHostCardDetect> {
        self.cd.as_ref()
    }

    pub fn max_freq(&self) -> u32 {
        self.max_freq
    }

    pub fn capbility(&self) -> u32 {
        self.capbility
    }
}
