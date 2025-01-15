#![allow(unused)] 

use core::{ptr::NonNull,time::Duration};
use bitflags::{bitflags, Flags};
use crate::{constants::{FSDIF_BUS_MODE_OFFSET, FSDIF_CARD_DETECT_OFFSET, FSDIF_CARD_RESET_OFFSET, FSDIF_CARD_THRCTL_OFFSET, FSDIF_CARD_WRTPRT_OFFSET, FSDIF_CKSTS_OFFSET, FSDIF_CLKDIV_OFFSET, FSDIF_CLKENA_OFFSET, FSDIF_CLK_SRC_OFFSET, FSDIF_CMD_OFFSET, FSDIF_CNTRL_OFFSET, FSDIF_CTYPE_OFFSET, FSDIF_DESC_LIST_ADDRH_OFFSET, FSDIF_DESC_LIST_ADDRL_OFFSET, FSDIF_DMAC_INT_EN_OFFSET, FSDIF_DMAC_STATUS_OFFSET, FSDIF_EMMC_DDR_REG_OFFSET, FSDIF_FIFOTH_OFFSET, FSDIF_INT_MASK_OFFSET, FSDIF_MASKED_INTS_OFFSET, FSDIF_PWREN_OFFSET, FSDIF_RAW_INTS_OFFSET, FSDIF_STATUS_OFFSET, FSDIF_TMOUT_OFFSET, FSDIF_UHS_REG_OFFSET}, err::{FsdifError, FsdifResult}, sleep};

/*
 * Create a contiguous bitmask starting at bit position @l and ending at
 * position @h. For example
 * GENMASK_ULL(39, 21) gives us the 64bit vector 0x000000ffffe00000.
 */
#[macro_export]
macro_rules! genmask {
    ($h:expr, $l:expr) => {
        (((!0u32) - (1u32 << $l) + 1) & ((!0u32) >> (32 - 1 - $h)))
    };
}

#[macro_export]
macro_rules! genmask_ull {
    ($h:expr, $l:expr) => {
        (((!0u64) - (1u64 << $l) + 1) & ((!0u64) >> (64 - 1 - $h)))
    };
}

/* set 32-bit register [a:b] as x, where a is high bit, b is low bit, x is setting/getting value */
#[macro_export]
macro_rules! get_reg32_bits {
    ($reg:expr, $a:expr, $b:expr) => {
        ($reg & genmask!($a, $b)) >> $b
    };
}

#[macro_export]
macro_rules! set_reg32_bits {
    ($reg:expr, $a:expr, $b:expr) => {
        (($reg << $b) & genmask!($a, $b))
    };
}

#[derive(Clone, Copy)]
pub struct Reg {
    pub addr: NonNull<u8>,
}

impl Reg {
    pub fn new(addr: NonNull<u8>) -> Self {
        Self { addr }
    }

    pub fn read_32(&self, reg: u32) -> u32 {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().read_volatile()
        }
    }

    pub fn write_32(&self, reg: u32, val: u32) {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().write_volatile(val);
        }
    }

    pub fn read_reg<F: FlagReg>(&self) -> F {
        F::from_bits_retain(self.read_32(F::REG))
    }

    pub fn write_reg<F: FlagReg>(&self, val: F) {
        self.write_32(F::REG, val.bits())
    }

    pub fn modify_reg<F: FlagReg>(&self, f: impl Fn(F) -> F) {
        let old = self.read_reg::<F>();
        self.write_reg(f(old));
    }

    pub fn wait_for<R: FlagReg, F: Fn(R) -> bool>(
        &self,
        f: F,
        interval: Duration,
        try_count: Option<usize>,
    ) -> FsdifResult {
        for _ in 0..try_count.unwrap_or(usize::MAX) {
            if f(self.read_reg::<R>()) {
                return Ok(());
            }

            sleep(interval);
        }
        Err(FsdifError::Timeout)
    }
    
}

pub trait FlagReg: Flags<Bits = u32> {
    const REG: u32;
}

// FSDIF_CNTRL_OFFSET x0 Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct FsdifCtrl: u32 {
        const CONTROLLER_RESET = 1 << 0; // RW 复位控制器，除 DMA，FIFO
        const FIFO_RESET = 1 << 1; // RW 复位 FIFO, 1 有效
        const DMA_RESET = 1 << 2; // RW 复位内部 DMA, 1 有效
        const INT_ENABLE = 1 << 4; // RW 全局中断使能配置, 1 使能
        const DMA_ENABLE = 1 << 5; // RW 外部 DMA 模式使能
        const READ_WAIT = 1 << 6; // RW SDIF 读等待 1 有效
        const SEND_IRQ_RESPONSE = 1 << 7; // RW MMC 中断自动响应配置 1 有效
        const ABORT_READ_DATA = 1 << 8; // RW 读暂停异常清除
        const SEND_CCSD = 1 << 9; // RW 发送CCD (NOT USED)
        const SEND_AUTO_STOP_CCSD = 1 << 10; // RW 发送CCD，自动STOP (NOT USED)
        const ENDIAN = 1 << 11; // RW 0：小端，1：大端
        const CARD_VOLTAGE_A_MASK = 0xf << 16; // RW A电压选择
        const CARD_VOLTAGE_B_MASK = 0xf << 20; // RW B电压选择
        const ENABLE_OD_PULLUP = 1 << 24; // RW 外部开漏输出
        const USE_INTERNAL_DMAC = 1 << 25; // RW 使用内部DMA
    }
}

impl FlagReg for FsdifCtrl {
    const REG: u32 = FSDIF_PWREN_OFFSET;
}

// FSDIF_PWREN_OFFSET 0x4 Register
bitflags! {
    pub struct FsdifPwrEn: u32 {
        const ENABLE = 1 << 0; // RW 卡供电开关, 0：关；1：开
    }
}

impl FlagReg for FsdifPwrEn {
    const REG: u32 = FSDIF_CNTRL_OFFSET;
}

// FSDIF_CLKDIV_OFFSET 0x8 Register
/* CLK_SAMPLE 和 CLK_SAMPLE 必须小于 CLK_DIVIDER */
bitflags! {
    pub struct FsdifClkDiv: u32 {
        const CLK_DIVIDER = 0x3ff;
        const CLK_SAMPLE_H = 1 << 23;
        const CLK_SAMPLE_L = 1 << 16;
        const CLK_DRV_H = 1 << 15;
        const CLK_DRV_L = 1 << 8;
        const CLK_DIVDER = 1 << 7;
        const CLK_DIVDER_EN = 1 << 0;
    }
}

impl FlagReg for FsdifClkDiv {
    const REG: u32 = FSDIF_CLKDIV_OFFSET;
}

pub fn clk_sample_set(reg:Reg,x:u32){
    reg.modify_reg::<FsdifClkDiv>(|reg| {
        reg | FsdifClkDiv::from_bits_truncate(set_reg32_bits!(x, 23, 16))
    });
}

pub fn clk_drv_set(reg:Reg,x:u32){
    reg.modify_reg::<FsdifClkDiv>(|reg| {
        reg | FsdifClkDiv::from_bits_truncate(set_reg32_bits!(x, 15, 8))
    });
}

pub fn clk_divider_set(reg:Reg,x:u32){
    reg.modify_reg::<FsdifClkDiv>(|reg| {
        reg | FsdifClkDiv::from_bits_truncate(set_reg32_bits!(x, 7, 0))
    });
}

pub fn clk_div(reg:Reg,samp:u32,drv:u32,div:u32){
    clk_sample_set(reg,samp);
    clk_drv_set(reg,drv);
    clk_divider_set(reg,div);
}

pub fn clk_divider_get(reg:Reg) -> u32 {
    (reg.read_reg::<FsdifClkDiv>() & FsdifClkDiv::CLK_DIVIDER).bits()
}

// FSDIF_CLKENA_OFFSET Register
bitflags! {
    pub struct FsdifClkEn: u32 {
        const CCLK_ENABLE = 1 << 0; /* RW 0：Clock disabled；1：Clock enabled */
        const CLKENA_CCLK_LOW_POWER = 1<<16; /* RW 0x0：非低功耗；0x1：低功耗 */
    }
}

impl FlagReg for FsdifClkEn {
    const REG: u32 = FSDIF_CLKENA_OFFSET;
}

// FSDIF_TMOUT_OFFSET Register
bitflags! {
    pub struct FsdifTimeout: u32 {
        const MAX_DATA_TIMEOUT = 0xffffff; /* RW 读卡超时（以卡时钟为单位） */
        const MAX_RESP_TIMEOUT = 0xff; /* RW 响应超时（以卡时钟为单位） */
    }
}

impl FlagReg for FsdifTimeout {
    const REG: u32 = FSDIF_TMOUT_OFFSET;
}

fn timeout_set(reg:Reg,data_timeout:u32,resp_timeout:u32){
    if data_timeout > FsdifTimeout::MAX_DATA_TIMEOUT.bits() {
        panic!("data_timeout is too large");
    }
    if resp_timeout > FsdifTimeout::MAX_RESP_TIMEOUT.bits() {
        panic!("resp_timeout is too large");
    }
    reg.write_reg(
        FsdifTimeout::from_bits_truncate(
        (genmask!(31,8) & (data_timeout << 8)) |
        (genmask!(7,0) & resp_timeout)
        )
    );
}

// FSDIF_CTYPE_OFFSET Register
bitflags! {
    pub struct FsdifCType: u32 {
        const CARD0_WIDTH1_8BIT = 1 << 16; /* 1: 8-bit mode */
        const CARD0_WIDTH2_4BIT = 1 << 0; /* 1: 4-bit mode */
        const CARD0_WIDTH2_1BIT = 0; /* 0: 1-bit mode */
    }
}

impl FlagReg for FsdifCType {
    const REG: u32 = FSDIF_CTYPE_OFFSET;
}

// FSDIF_INT_MASK_OFFSET Register
bitflags! {
    pub struct FsdifInt: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        const INTS_CMD_MASK = 0x1546;
        const INTS_DATA_MASK = 0x2288;
    }
}

impl FlagReg for FsdifInt {
    const REG: u32 = FSDIF_INT_MASK_OFFSET; // 假设FSDIF_INT_OFFSET是对应的寄存器偏移量
}

// FSDIF_MASKED_INTS_OFFSET Register
bitflags! {
    pub struct FsdifMaskedInts: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        const INTS_CMD_MASK = 0x1546;
        const INTS_DATA_MASK = 0x2288;
    }
}

impl FlagReg for FsdifMaskedInts {
    const REG: u32 = FSDIF_MASKED_INTS_OFFSET;
}

// FSDIF_RAW_INTS_OFFSET Register
bitflags! {
    pub struct FsdifRawInts: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        const INTS_CMD_MASK = 0x1546;
        const INTS_DATA_MASK = 0x2288;
    }
}

impl FlagReg for FsdifRawInts {
    const REG: u32 = FSDIF_RAW_INTS_OFFSET;
}

// FSDIF_CMD_OFFSET Register
bitflags! {
    pub struct FsdifCmd: u32 {
        const START = 1 << 31;                /* 启动命令 */
        const USE_HOLD_REG = 1 << 29;         /* 0: 旁路HOLD寄存器，1: 使能HOLD寄存器 */
        const VOLT_SWITCH = 1 << 28;          /* 0: 无电压转换，1: 有电压转换 */
        const BOOT_MODE = 1 << 27;            /* 0: Mandatory boot, 1: Alternate boot */
        const DISABLE_BOOT = 1 << 26;         /* 中止boot进程 */
        const EXPECT_BOOT_ACK = 1 << 25;      /* 1: Expect boot ack */
        const ENABLE_BOOT = 1 << 24;          /* 1: 使能 boot for mandatory */
        const UPD_CLK = 1 << 21;              /* 1：不发送指令，仅更新时钟寄存器的值到卡时钟域内 */
        const INIT = 1 << 15;                  /* 0：在发送指令前不发送初始化序列（80 个周期） 1: 发送 */
        const STOP_ABORT = 1 << 14;           /* 1：停止或中止命令，用于停止当前的数据传输 */
        const WAIT_PRVDATA_COMPLETE = 1 << 13; /* 1：等待前面的数据传输完成后再发送指令 0: 立即发送命令 */
        const SEND_AUTO_STOP = 1 << 12;       /* 1：在数据传送结束时发送停止命令 */
        const DAT_WRITE = 1 << 10;            /* 0：读卡 1：写卡 */
        const DAT_EXP = 1 << 9;                /* 0：不等待数据传输, 1：等待数据传输 */
        const RESP_CRC = 1 << 8;               /* 1：检查响应 CRC */
        const RESP_LONG = 1 << 7;              /* 0：等待卡的短响应 1：等待卡的长响应 */
        const RESP_EXP = 1 << 6;               /* 1：等待卡的响应，0：命令不需要卡响应 */
    }
}

impl FlagReg for FsdifCmd {
    const REG: u32 = FSDIF_CMD_OFFSET; // 假设FSDIF_CMD_OFFSET是对应的寄存器偏移量
}

impl From<u32> for FsdifCmd {
    fn from(val: u32) -> Self {
        FsdifCmd::from_bits_truncate(val)
    }
}

pub fn cmd_card_num_set(reg: Reg, num: u32) {
    reg.modify_reg::<FsdifCmd>(|reg| {
        reg | FsdifCmd::from_bits_truncate(set_reg32_bits!(num, 20, 16))
    });
}

/* 1: 流数据传输指令 */
pub fn cmd_transf_mode_set(reg: Reg, mode: u32) {
    reg.modify_reg::<FsdifCmd>(|reg| {
        reg | FsdifCmd::from_bits_truncate(set_reg32_bits!(mode, 12, 11))
    });
}

/* 命令索引号 */
pub fn cmd_indx_set(reg: Reg, ind: u32) {
    reg.modify_reg::<FsdifCmd>(|reg| {
        reg | FsdifCmd::from_bits_truncate(set_reg32_bits!(ind, 5, 0))
    });
}

pub fn cmd_indx_get(reg: Reg) -> u32 {
    (reg.read_reg::<FsdifCmd>() & FsdifCmd::from_bits_truncate(genmask!(5, 0))).bits()
}

// FSDIF_STATUS_OFFSET Register
bitflags! {
    pub struct FsdifStatus: u32 {
        const FIFO_RX = 1 << 0;     /* RO, 达到 FIFO_RX 标记 */
        const FIFO_TX = 1 << 1;     /* RO, 达到 FIFO_TX 标记 */
        const FIFO_EMPTY = 1 << 2;  /* RO, FIFO empty */
        const FIFO_FULL = 1 << 3;   /* RO, FIFO full */
        const DATA3_STATUS = 1 << 8; /* RO DATA[3] 卡在位检测，1：在位 */
        const DATA_BUSY = 1 << 9;   /* RO 1: 卡 busy */
        const DATA_STATE_MC_BUSY = 1 << 10;  /* RO DATA TX|RX FSM busy  */
        const DMA_ACK = 1 << 30;    /* RO DMA 确认 */
        const DMA_REQ = 1 << 31;    /* RO DMA 请求 */
    }
}

impl FlagReg for FsdifStatus {
    const REG: u32 = FSDIF_STATUS_OFFSET;
}

pub fn cmd_fsm_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifStatus>().bits(), 7, 4)
}

pub fn resp_index_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifStatus>().bits(), 16, 11)
}

pub fn fifo_cnt_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifStatus>().bits(), 29, 17)
}

// FSDIF_FIFOTH_OFFSET Register
bitflags! {
    pub struct FsdifFifoTh: u32 {
        const DMA_TRANS_MASK = genmask!(30, 28); /* 多次传输的突发大小 */
        const RX_WMARK_MASK = genmask!(27, 16);  /* 当接收数据给卡时FIFO的阈值 */
        const TX_WMARK_MASK = genmask!(11, 0);   /* 当发送数据给卡时FIFO的阈值 */
    }
}

impl FlagReg for FsdifFifoTh {
    const REG: u32 = FSDIF_FIFOTH_OFFSET;
}

impl From<u32> for FsdifFifoTh {
    fn from(val: u32) -> Self {
        FsdifFifoTh::from_bits_truncate(val)
    }
}

pub enum FsdifFifoThDmaTransSize {
    DmaTrans1 = 0b000,
    DmaTrans4 = 0b001,
    DmaTrans8 = 0b010,
    DmaTrans16 = 0b011,
    DmaTrans32 = 0b100,
    DmaTrans64 = 0b101,
    DmaTrans128 = 0b110,
    DmaTrans256 = 0b111
}

impl From<FsdifFifoThDmaTransSize> for u32 {
    fn from(val: FsdifFifoThDmaTransSize) -> Self {
        val as u32
    }
}

pub const FSDIF_RX_WMARK:u32 = 0x7;
pub const FSDIF_TX_WMARK:u32 = 0x100;

pub fn dma_trans_size_set(reg:Reg,size:u32){
    reg.modify_reg::<FsdifFifoTh>(|reg| {
        reg | FsdifFifoTh::from_bits_truncate(set_reg32_bits!(size, 30, 28))
    });
}

pub fn rx_mark_size_set(reg:Reg,size:u32){
    reg.modify_reg::<FsdifFifoTh>(|reg| {
        reg | FsdifFifoTh::from_bits_truncate(set_reg32_bits!(size, 27, 16))
    });
}

pub fn tx_mark_size_set(reg:Reg,size:u32){
    reg.modify_reg::<FsdifFifoTh>(|reg| {
        reg | FsdifFifoTh::from_bits_truncate(set_reg32_bits!(size, 11, 0))
    });
}

/// FSDIF_CARD_DETECT_OFFSET Register
bitflags! {
    pub struct FsdifCardDetect: u32 {
        const DETECTED = 1 << 0; /* 1：卡不在位；0：卡在位 */
    }
}

impl FlagReg for FsdifCardDetect {
    const REG: u32 = FSDIF_CARD_DETECT_OFFSET; // 假设 FSDIF_CARD_DETECT_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_CARD_WRTPRT_OFFSET Register
bitflags! {
    pub struct FsdifCardWrtp: u32 {
        const WRITE_PROTECTED = 1 << 0; /* 1：写保护；0：无写保护 */
    }
}

impl FlagReg for FsdifCardWrtp {
    const REG: u32 = FSDIF_CARD_WRTPRT_OFFSET; // 假设 FSDIF_CARD_WRTPRT_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_CKSTS_OFFSET Register
bitflags! {
    pub struct FsdifClkSts: u32 {
        const READY = 1 << 0; /* CIU 时钟 ready */
    }
}

impl FlagReg for FsdifClkSts {
    const REG: u32 = FSDIF_CKSTS_OFFSET; // 假设 FSDIF_CKSTS_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_UHS_REG_OFFSET Register
bitflags! {
    pub struct FsdifUhsReg: u32 {
        const VOLT_180 = 1 << 0; /* RW 外部调压器接口电压 0: 3.3v, 1: 1.8v */
        const DDR = 1 << 16;     /* RW DDR 模式 */
    }
}

impl FlagReg for FsdifUhsReg {
    const REG: u32 = FSDIF_UHS_REG_OFFSET; // 假设 FSDIF_UHS_REG_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_CARD_RESET_OFFSET Register
bitflags! {
    pub struct FsdifCardReset: u32 {
        const ENABLE = 1 << 0; /* RW 1：运行；0：复位 */
    }
}

impl FlagReg for FsdifCardReset {
    const REG: u32 = FSDIF_CARD_RESET_OFFSET; // 假设 FSDIF_CARD_RESET_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_BUS_MODE_OFFSET Register
bitflags! {
    pub struct FsdifBusMode: u32 {
        const SWR = 1 << 0; /* RW 软复位，复位idma内部寄存器 */
        const FB = 1 << 1;  /* RW 固定burst */
        const DE = 1 << 7;  /* RW idma使能 */
    }
}

impl FlagReg for FsdifBusMode {
    const REG: u32 = FSDIF_BUS_MODE_OFFSET; // 假设 FSDIF_BUS_MODE_OFFSET 是对应的寄存器偏移量
}

pub fn bus_mode_pbl_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifBusMode>().bits(), 10, 8)
}

/// FSDIF_DMAC_STATUS_OFFSET Register
bitflags! {
    pub struct FsdifDmacStatus: u32 {
        const TI = 1 << 0;  /* RW 发送中断。表示链表的数据发送完成 */
        const RI = 1 << 1;  /* RW 接收中断。表示链表的数据接收完成 */
        const FBE = 1 << 2; /* RW 致命总线错误中断 */
        const DU = 1 << 4;  /* RW 链表不可用中断 */
        const CES = 1 << 5; /* RW 卡错误汇总 */
        const NIS = 1 << 8; /* RW 正常中断汇总 */
        const AIS = 1 << 9; /* RW 异常中断汇总 */
        const ALL_BITS = 0x3ff;
        const STATUS_EB_TX = 0b001;
        const STATUS_EB_RX = 0b010;
    }
}

impl FlagReg for FsdifDmacStatus {
    const REG: u32 = FSDIF_DMAC_STATUS_OFFSET; // 假设 FSDIF_DMAC_STATUS_OFFSET 是对应的寄存器偏移量
}

pub fn dmac_status_eb_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifDmacStatus>().bits(), 12, 10)
}

/// FSDIF_DMAC_INT_EN_OFFSET Register
bitflags! {
    pub struct FsdifDmacIntEn: u32 {
        const TI = 1 << 0;  /* RW 发送完成中断使能 */
        const RI = 1 << 1;  /* RW 接收完成中断使能 */
        const FBE = 1 << 2; /* RW 总线错误中断使能 */
        const DU = 1 << 4;  /* RW 描述符不可读中断使能 */
        const CES = 1 << 5; /* RW 卡错误中断使能 */
        const NIS = 1 << 8; /* RW 正常中断汇总使能 */
        const AIS = 1 << 9; /* RW 异常中断汇总使能 */
        const ALL_BITS = 0x3ff;
        const INTS_MASK = 0x314;
    }
}

impl FlagReg for FsdifDmacIntEn {
    const REG: u32 = FSDIF_DMAC_INT_EN_OFFSET; // 假设 FSDIF_DMAC_INT_EN_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_CARD_THRCTL_OFFSET Register
bitflags! {
    pub struct FsdifCardThrctl: u32 {
        const CARDRD = 1 << 0;   /* RW 读卡threshold使能 */
        const BUSY_CLR = 1 << 1; /* RW busy清中断 */
        const CARDWR = 1 << 2;   /* RO 写卡threshold使能 */
    }
}

impl FlagReg for FsdifCardThrctl {
    const REG: u32 = FSDIF_CARD_THRCTL_OFFSET; // 假设 FSDIF_CARD_THRCTL_OFFSET 是对应的寄存器偏移量
}

impl From<u32> for FsdifCardThrctl {
    fn from(val: u32) -> Self {
        FsdifCardThrctl::from_bits_truncate(val)
    }
}

pub enum FsdifFifoDepth {
    Depth8 = 23,
    Depth16 = 24,
    Depth32 = 25,
    Depth64 = 26,
    Depth128 = 27,
}

impl FsdifFifoDepth {
    pub fn card_thrctl_threshold(self) -> u32 {
        1 << self as u32
    }
}

// 读卡 Threshold
pub fn card_thrctl_threshold(reg:Reg,n: u32) -> u32 {
    reg.read_reg::<FsdifCardThrctl>().bits() & (1<<n)
}

/// FSDIF_CLK_SRC_OFFSET Register
bitflags! {
    pub struct FsdifClkSrc: u32 {
        const UHS_EXT_MMC_VOLT = 1 << 0;         /* RW 1.2V供电选择 */
        const UHS_EXT_CLK_ENA = 1 << 1;          /* RW 外部时钟，CIU时钟使能 */
        const UHS_EXT_CLK_MUX = 1 << 31;         /* RW 外部时钟选择 */
        const UHS_CLK_DIV_MASK = genmask!(14, 8); /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_SAMP_MASK = genmask!(22, 16); /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_MASK = genmask!(30, 24); /* RW 输出相位参数，相对于ciu时钟相位点 */
    }
}

impl FlagReg for FsdifClkSrc {
    const REG: u32 = FSDIF_CLK_SRC_OFFSET; // 假设 FSDIF_CLK_SRC_OFFSET 是对应的寄存器偏移量
}

pub fn uhs_clk_div(x: u32) -> FsdifClkSrc {
    FsdifClkSrc::UHS_CLK_DIV_MASK & FsdifClkSrc::from_bits_truncate(x << 8)
}

pub fn uhs_clk_samp(x: u32) -> FsdifClkSrc {
    FsdifClkSrc::UHS_CLK_SAMP_MASK & FsdifClkSrc::from_bits_truncate(x << 16)
}

pub fn uhs_clk_drv(x: u32) -> FsdifClkSrc {
    FsdifClkSrc::UHS_CLK_DRV_MASK & FsdifClkSrc::from_bits_truncate(x << 24)
}

pub fn uhs_reg(drv_phase: u32, samp_phase: u32, clk_div: u32) -> FsdifClkSrc {
    uhs_clk_div(clk_div) | uhs_clk_samp(samp_phase) | uhs_clk_drv(drv_phase)
}

pub fn uhs_clk_div_set(reg: Reg, x: u32) {
    reg.modify_reg::<FsdifClkSrc>(|reg| {
        reg | FsdifClkSrc::from_bits_truncate(set_reg32_bits!(x,14,8))
    });
}

pub fn uhs_clk_div_get(reg: Reg) -> u32 {
    get_reg32_bits!(reg.read_reg::<FsdifClkSrc>().bits(),14,8)
}

pub fn uhs_clk_samp_set(reg: Reg, x: u32) {
    reg.modify_reg::<FsdifClkSrc>(|reg| {
        reg | FsdifClkSrc::from_bits_truncate(set_reg32_bits!(x,22,16))
    });
}

pub fn uhs_clk_drv_set(reg: Reg, x: u32) {
    reg.modify_reg::<FsdifClkSrc>(|reg| {
        reg | FsdifClkSrc::from_bits_truncate(set_reg32_bits!(x,30,24))
    });
}

/// FSDIF_EMMC_DDR_REG_OFFSET Register
bitflags! {
    pub struct FsdifEmmcDdrReg: u32 {
        const CYCLE = 1 << 0; /* RW 1: start bit小于一个周期，0：start bit 为一个周期 */
    }
}

impl FlagReg for FsdifEmmcDdrReg {
    const REG: u32 = FSDIF_EMMC_DDR_REG_OFFSET; // 假设 FSDIF_EMMC_DDR_REG_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_DESC_LIST_ADDRH_OFFSET Register
bitflags! {
    pub struct FsdifDescListAddrH: u32 {
        
    }
}

impl FlagReg for FsdifDescListAddrH {
    const REG: u32 = FSDIF_DESC_LIST_ADDRH_OFFSET; // 假设 FSDIF_DESC_LIST_ADDRH_OFFSET 是对应的寄存器偏移量
}

/// FSDIF_DESC_LIST_ADDRL_OFFSET Register
bitflags! {
    pub struct FsdifDescListAddrL: u32 {
        
    }
}

impl FlagReg for FsdifDescListAddrL {
    const REG: u32 = FSDIF_DESC_LIST_ADDRL_OFFSET; // 假设 FSDIF_DESC_LIST_ADDRL_OFFSET 是对应的寄存器偏移量
}