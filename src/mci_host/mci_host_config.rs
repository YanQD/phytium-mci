use crate::mci::constants::MCIId;

pub struct MCIHostConfig {
    host_id: MCIId,                     // 主机 ID
    host_type: MCIHostType,           // 主机类型
    card_type: MCIHostCardType,       // 卡类型
    enable_irq: bool,                 // 是否启用中断
    enable_dma: bool,                 // 是否启用 DMA
    endian_mode: MCIHostEndianMode,   // 字节序模式
    max_trans_size: usize,            // 最大传输大小
    def_block_size: usize,            // 默认块大小
    card_clock: u32,                  // 卡时钟频率
    is_uhs_card: bool,                // 是否为 UHS 卡
    /* for SDIO card, to support card customized interrupt handling */ // todo 暂时没实现这部分功能
}

impl MCIHostConfig {
    pub fn host_id(&self) -> MCIId {
        self.host_id
    }

    pub fn host_type(&self) -> MCIHostType {
        self.host_type
    }

    pub fn card_type(&self) -> MCIHostCardType {
        self.card_type
    }

    pub fn enable_irq(&self) -> bool {
        self.enable_irq
    }

    pub fn enable_dma(&self) -> bool {
        self.enable_dma
    }

    pub fn endian_mode(&self) -> MCIHostEndianMode {
        self.endian_mode
    }

    pub fn max_trans_size(&self) -> usize {
        self.max_trans_size
    }

    pub fn def_block_size(&self) -> usize {
        self.def_block_size
    }

    pub fn card_clock(&self) -> u32 {
        self.card_clock
    }

    pub fn is_uhs_card(&self) -> bool {
        self.is_uhs_card
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostType {
    SDMMC,
    SDIF
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostCardType {
    StandardSD,
    MicroSD,
    EMMC,
    SDIO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostEndianMode {
    Big = 0, /* Big endian mode */
    HalfWordBig = 1, /* Half word big endian mode */
    Little = 2, /* Little endian mode */
}
