use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use consts::MAX_POOL_SIZE;
use err::FMempStatus;
use log::{error, info, warn};
use rlsf::Tlsf;
use spin::Mutex;
use lazy_static::*;

mod err;
mod consts;

static mut POOL: [MaybeUninit<u8>; MAX_POOL_SIZE] = [MaybeUninit::uninit(); MAX_POOL_SIZE];

pub struct FMemp<'a> {
    tlsf_ptr: Tlsf<'a, u32, u32, 32, 32>,
    is_ready: bool,
}

lazy_static! {
    pub static ref GLOBAL_FMEMP: Mutex<FMemp<'static>> = 
        Mutex::new(FMemp::new());
}

impl<'a> FMemp<'a> {
    pub fn new() -> Self {
        Self {
            tlsf_ptr: Tlsf::new(),
            is_ready: false,
        }
    }

    unsafe fn init(&mut self) {
        self.tlsf_ptr.insert_free_block(&mut POOL[..]);
        self.is_ready = true;
    }

    unsafe fn alloc_aligned(&mut self, size: usize, align: usize) -> NonNull<u8> {
        let layout = Layout::from_size_align_unchecked(size, align);
        self.tlsf_ptr.allocate(layout).unwrap()
    }
}

/// 初始化内存池 大小为1MiB
pub unsafe fn osa_init() {
    GLOBAL_FMEMP.lock().init();
}

/// 分配'size'大小的空间，默认64KiB对齐
pub unsafe fn osa_alloc(size: usize) -> NonNull<u8> {
    GLOBAL_FMEMP.lock().alloc_aligned(size, size_of::<usize>())
}

/// 申请'size'大小的空间，对齐到'align'
pub unsafe fn osa_alloc_aligned(size: usize, align: usize) -> NonNull<u8> {
    GLOBAL_FMEMP.lock().alloc_aligned(size, align)
}