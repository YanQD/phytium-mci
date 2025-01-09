#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bare_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::time::Duration;

use bare_test::{
    driver::device_tree::get_device_tree, fdt::PciSpace, mem::mmu::iomap, println, time::delay,
};
use log::*;

bare_test::test_setup!();

#[test_case]
fn test_work() {
    assert!(true);
}

fn sleep(duration: Duration) {
    spin_on::spin_on(delay(duration));
}

struct KernelImpl;

impl sdif_driver::Kernel for KernelImpl {
    fn sleep(duration: Duration) {
        sleep(duration);
    }
}

sdif_driver::set_impl!(KernelImpl);
