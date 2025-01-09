#![no_std]

extern crate alloc;

use core::time::Duration;

mod sd;

pub use sd::*;

pub trait Kernel {
    fn sleep(duration: Duration);
}

pub(crate) fn sleep(duration: Duration) {
    extern "Rust" {
        fn _sdif_driver_sleep(duration: Duration);
    }

    unsafe {
        _sdif_driver_sleep(duration);
    }
}

#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _sdif_driver_sleep(duration: core::time::Duration) {
            <$t as $crate::Kernel>::sleep(duration)
        }
    };
}
