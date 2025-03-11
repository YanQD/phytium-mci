use super::constants::MCIHostDetectCardType;

pub struct MCIHostCardDetect {
    typ:MCIHostDetectCardType,
    cd_debounce_ms: u32,
    // todo Function type that takes a boolean and a user data pointer
    card_detected: Option<MCIHostCdStatusFn>,
    dat3_pull_func: Option<MCIHostDat3PullFn>,
    // todo user data
}

type MCIHostCdStatusFn = fn() -> bool;
type MCIHostDat3PullFn = fn(pull_status: u32);

impl MCIHostCardDetect {
    pub fn typ(&self) -> MCIHostDetectCardType {
        self.typ
    }
    
    pub fn cd_debounce_ms(&self) -> u32 {
        self.cd_debounce_ms
    }

    pub fn card_detected(&self) -> Option<&MCIHostCdStatusFn> {
        self.card_detected.as_ref()
    }
}