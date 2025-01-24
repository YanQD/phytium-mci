use crate::{mci_timing::*, FsDifClkSpeed, FsDifTransMode};

type GetTuning = fn(clock_freq: FsDifClkSpeed, non_removable: bool) -> MCITiming;
pub struct MCIConfig {
    pub instance_id: u32,           /* Device instance id */
    pub irq_num: u32,               /* Device IRQ number */
    pub trans_mode: FsDifTransMode, /* Trans mode, PIO/DMA */
    pub non_removable: bool,        /* Non-removable media, e.g. eMMC */
    pub get_tuning: GetTuning,
}


impl MCIConfig {
    pub fn new() -> Self {
        MCIConfig {
            instance_id: 0,
            irq_num: 104,
            trans_mode: FsDifTransMode::PioTransMode,
            non_removable: false,
            get_tuning: default_tuning,
        }
    }
}

pub fn default_tuning(_clock_freq: FsDifClkSpeed, _non_removable: bool) -> MCITiming {
    MMC_SD_NULL
}

fn MCIGetTimingSetting(clock_freq: FsDifClkSpeed, non_removable: bool) -> MCITiming {
    
}