use crate::mci_host::constants::MCIHostOperationVoltage;

use super::constants::SdIoVoltageCtrlType;

pub(crate) struct SdIoVoltage {
    typ: SdIoVoltageCtrlType,
    func: Option<SdIoVoltageFn>,
}

type SdIoVoltageFn = fn(MCIHostOperationVoltage);

impl SdIoVoltage {
    pub(crate) fn typ(&self) -> SdIoVoltageCtrlType {
        self.typ
    }

    pub(crate) fn func(&self) -> Option<&SdIoVoltageFn> {
        self.func.as_ref()
    }
}