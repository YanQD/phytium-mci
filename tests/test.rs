#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bare_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::time::Duration;

use bare_test::{driver::device_tree::get_device_tree, mem::mmu::iomap, println, time::delay};
use log::*;
use phytium_mci::{iopad::PAD_ADDRESS, *};

bare_test::test_setup!();

#[test_case]
fn test_work() {
    let fdt = get_device_tree().unwrap();

    let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();

    let reg = mci0.reg().unwrap().next().unwrap();

    info!("mci0 reg: {:#x},mci0 reg size: {:#x}", reg.address, reg.size.unwrap());

    let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

    let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);

    let iopad = IoPad::new(iopad_reg_base);

    let sdcard = SdCard::example_instance(mci_reg_base);

    assert!(true);
}

fn sleep(duration: Duration) {
    spin_on::spin_on(delay(duration));
}

struct KernelImpl;

impl Kernel for KernelImpl {
    fn sleep(duration: Duration) {
        sleep(duration);
    }
}

set_impl!(KernelImpl);
