#![no_std]

extern crate alloc;

use core::time::Duration;

#[macro_use]
mod regs;
pub mod iopad;
pub mod mci;
pub mod mci_host;
pub mod osa;
mod tools;

pub use iopad::*;
pub use mci_host::*;

// pub use dma_api::{set_impl as set_dma_impl, Direction as DmaDirection, Impl as DmaImpl};

pub trait Kernel {
    fn sleep(duration: Duration);
}

pub(crate) fn sleep(duration: Duration) {
    extern "Rust" {
        fn _phytium_mci_sleep(duration: Duration);
    }

    unsafe {
        _phytium_mci_sleep(duration);
    }
}

#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _phytium_mci_sleep(duration: Duration) {
            <$t as $crate::Kernel>::sleep(duration)
        }
    };
}
