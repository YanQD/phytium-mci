use crate::mci::FsDifClkSpeed;

type PadDelay = fn(id:u32);

pub struct MCITiming {
    pub use_hold: bool,
    pub clk_div: u32,
    pub clk_src: u32,
    pub shift: u32,
    pub pad_delay: PadDelay //* 用于调整IO的延时 */
}

impl MCITiming {
    pub fn new() -> Self {
        MCITiming {
            use_hold: false,
            clk_div: 0,
            clk_src: 0,
            shift: 0,
        }
    }
}

pub const MMC_SD_NULL: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0,
    clk_src: 0,
    shift: 0,
};

pub const MMC_SD_400K_HZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x7e7dfa,
    clk_src: 0x000502,
    shift: 0x0,
};

pub const SD_25MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000302,
    shift: 0x0,
};

pub const SD_50MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000502,
    shift: 0x0,
};

pub const SD_100MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
};

pub const MMC_26MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000302,
    shift: 0x0,
};

pub const MMC_52MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x030204,
    clk_src: 0x000202,
    shift: 0x0,
};

pub const MMC_66MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
};

pub const MMC_100MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
};

pub fn default_tuning(_clock_freq: FsDifClkSpeed, _non_removable: bool) -> &'static MCITiming {
    &MMC_SD_NULL
}

pub fn mciget_timing_setting(clock_freq: FsDifClkSpeed, non_removable: bool) -> &'static MCITiming {
    if clock_freq == FsDifClkSpeed::ClkSpeed400KHz {
        return &MMC_SD_400K_HZ;
    }
    if non_removable {
        match clock_freq {
            FsDifClkSpeed::ClkSpeed25Mhz => return &SD_25MHZ,
            FsDifClkSpeed::ClkSpeed50Mhz => return &SD_50MHZ,
            FsDifClkSpeed::ClkSpeed100Mhz => return &SD_100MHZ,
            _ => return &MMC_SD_NULL,
        }
    }else {
        match clock_freq {
            FsDifClkSpeed::ClkSpeed26Mhz => return &MMC_26MHZ,
            FsDifClkSpeed::ClkSpeed52Mhz => return &MMC_52MHZ,
            FsDifClkSpeed::ClkSpeed66Mhz => return &MMC_66MHZ,
            FsDifClkSpeed::ClkSpeed100Mhz => return &MMC_100MHZ,
            _ => return &MMC_SD_NULL,
        }
    }
}