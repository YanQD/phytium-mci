use core::ptr::NonNull;

pub struct SDIF {
    reg_base: usize,
}
impl SDIF {
    pub fn new(reg_base: NonNull<u8>) -> Self {
        SDIF {
            reg_base: reg_base.as_ptr() as usize,
        }
    }

    fn read(&self, offset: usize) -> u32 {
        unsafe {
            let ptr = self.reg_base + offset;
            (ptr as *const u32).read_volatile()
        }
    }

    pub fn vid(&self) -> u32 {
        self.read(0x6c)
    }

    pub fn uid(&self) -> u32 {
        self.read(0x68)
    }

    pub fn card_detected(&self) -> bool {
        self.read(0x50) == 0
    }

    pub fn blksize(&self) -> u32 {
        self.read(0x1c)
    }
}
