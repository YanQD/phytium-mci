use crate::{mci_timing::*, FsDifClkSpeed, FsDifTransMode};
trait MCIGetTuning {
    fn get_tuning(clock_freq:FsDifClkSpeed,non_removable:bool) -> MCITiming;
}
pub struct MCIConfig {
    pub instance_id: u32,           /* Device instance id */
    pub irq_num: u32,               /* Device IRQ number */
    pub trans_mode: FsDifTransMode, /* Trans mode, PIO/DMA */
    pub non_removable: bool,        /* Non-removable media, e.g. eMMC */
}

//?可变的GetTuning没有实现 ?/
impl MCIGetTuning for MCIConfig {
    fn get_tuning(clock_freq: FsDifClkSpeed, non_removable: bool) -> MCITiming {
        if clock_freq == FsDifClkSpeed::ClkSpeed400KHz {
            return MMC_SD_400K_HZ;
        }
        if non_removable {
            match clock_freq {
                FsDifClkSpeed::ClkSpeed26Mhz => return MMC_26MHZ,
                FsDifClkSpeed::ClkSpeed52Mhz => return MMC_52MHZ,
                FsDifClkSpeed::ClkSpeed66Mhz => return MMC_66MHZ,
                FsDifClkSpeed::ClkSpeed100Mhz => return MMC_100MHZ,
                FsDifClkSpeed::ClkSpeed400KHz => todo!(),
                FsDifClkSpeed::ClkSpeed25Mhz => todo!(),
                FsDifClkSpeed::ClkSpeed50Mhz => todo!(),
            }
        }else {
            match clock_freq {
                FsDifClkSpeed::ClkSpeed25Mhz => return SD_25MHZ,
                FsDifClkSpeed::ClkSpeed50Mhz => return  SD_50MHZ,
                FsDifClkSpeed::ClkSpeed100Mhz => return  SD_100MHZ,
                FsDifClkSpeed::ClkSpeed400KHz => todo!(),
                FsDifClkSpeed::ClkSpeed26Mhz => todo!(),
                FsDifClkSpeed::ClkSpeed52Mhz => todo!(),
                FsDifClkSpeed::ClkSpeed66Mhz => todo!(),
            }
        }
    }
}

impl MCIConfig {
    pub fn new() -> Self {
        MCIConfig {
            instance_id: 0,
            irq_num: 104,
            trans_mode: FsDifTransMode::PioTransMode,
            non_removable: false,
        }
    }
}