
pub(crate) struct MCIHostCardDetect {
    typ: DetectCardType,
    cd_debounce_ms: u32,
}

impl MCIHostCardDetect {
    pub(crate) fn cd_debounce_ms(&self) -> u32 {
        self.cd_debounce_ms
    }
}

// todo 前缀不对
pub(crate) enum DetectCardType {
    GpioCD,
    HostCD,
    HostDATA3
}