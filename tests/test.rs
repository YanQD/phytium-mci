#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::time::Duration;

use alloc::vec::Vec;
use bare_test::{driver::device_tree::get_device_tree, mem::mmu::iomap, print, time::delay};
use phytium_mci::sd::SdCard; // Adjusted to the correct module path
use log::*;
use phytium_mci::{iopad::PAD_ADDRESS, *};

        let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();

        let reg = mci0.reg().unwrap().next().unwrap();

        info!(
            "mci0 reg: {:#x},mci0 reg size: {:#x}",
            reg.address,
            reg.size.unwrap()
        );

        let reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

        let mut mci0 = MCI::new(reg_base);

    let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

    let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);

    let iopad = IoPad::new(iopad_reg_base);

    let mut sdcard = SdCard::example_instance(mci_reg_base,iopad);

    let mut buffer = Vec::new();
    let _ = sdcard.read_blocks(&mut buffer, 131072+100,1);

    print!("test_work passed\n");
    for i in 0..buffer.len() {
        warn!("{:x},{:x},{:x},{:x}",
                buffer[i] as u8,
                (buffer[i] >> 8) as u8,
                (buffer[i] >> 16) as u8,
                (buffer[i] >> 24) as u8);
    }
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
