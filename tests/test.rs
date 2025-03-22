#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::time::Duration;

    use bare_test::{
        globals::{global_val, PlatformInfoKind},
        mem::mmu::iomap,
        time::spin_delay,
    };
    use log::*;
    use phytium_mci::{iopad::PAD_ADDRESS, *};
    #[test]
    fn test_work() {
        
    }

    fn sleep(duration: Duration) {
        spin_delay(duration);
    }

    struct KernelImpl;

    impl Kernel for KernelImpl {
        fn sleep(duration: Duration) {
            sleep(duration);
        }
    }

    set_impl!(KernelImpl);
}