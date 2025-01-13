use core::{ptr::NonNull,time::Duration};
use bitflags::{bitflags, Flags};
use crate::{constants::{FSDIF_CLKDIV_OFFSET, FSDIF_CLKENA_OFFSET, FSDIF_CNTRL_OFFSET, FSDIF_PWREN_OFFSET}, err::FsdifError, sleep};

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
    ) -> Result<(), FsdifError> {
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
        const CCLK_ENABLE = 0; /* RW 0：Clock disabled；1：Clock enabled */
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

