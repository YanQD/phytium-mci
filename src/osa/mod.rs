//! An area managed by Tlsf algorithm
#![deny(missing_docs)]
use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use consts::MAX_POOL_SIZE;
use err::FMempError;
use rlsf::Tlsf;
use spin::Mutex;
use lazy_static::*;

mod err;
mod consts;
pub mod pool_buffer;

/// Memory menaged by Tlsf pool
static mut POOL: [MaybeUninit<u8>; MAX_POOL_SIZE] = [MaybeUninit::uninit(); MAX_POOL_SIZE];

/// Tlsf controller
pub struct FMemp<'a> {
    tlsf_ptr: Tlsf<'a, u32, u32, 32, 32>,
    is_ready: bool,
}

lazy_static! {
    /// Global memory pool manager
    pub static ref GLOBAL_FMEMP: Mutex<FMemp<'static>> = 
        Mutex::new(FMemp::new());
}

impl<'a> FMemp<'a> {
    /// Constructor
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

    unsafe fn alloc_aligned(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, FMempError> {
        let layout = Layout::from_size_align_unchecked(size, align);
        if let Some(result) = self.tlsf_ptr.allocate(layout) {
            Ok(result)
        } else {
            Err(FMempError::BadMalloc)
        }
    }

    unsafe fn dealloc(&mut self, addr: NonNull<u8>, size: usize) {
        self.tlsf_ptr.deallocate(addr, size);
    }
}  

/// Init memory pool with size of ['MAX_POOL_SIZE']
pub fn osa_init() {
    unsafe { GLOBAL_FMEMP.lock().init(); }
}

/// Alloc 'size' bytes space, aligned to 64 KiB by default
pub fn osa_alloc(size: usize) -> Result<NonNull<u8>, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, size_of::<usize>()) }
}

/// Alloc 'size' bytes space, aligned to 'align' bytes
pub fn osa_alloc_aligned(size: usize, align: usize) -> Result<NonNull<u8>, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, align) }
}

/// Dealloc 'size' bytes space from 'addr'
pub fn osa_dealloc(addr: NonNull<u8>, size: usize) {
    unsafe { GLOBAL_FMEMP.lock().dealloc(addr, size); }
}