#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bare_test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::time::Duration;

use bare_test::{driver::device_tree::get_device_tree, mem::mmu::iomap, println, time::delay};
use log::*;
use phytium_mci::*;

bare_test::test_setup!();

#[test_case]
fn test_work() {
    let fdt = get_device_tree().unwrap();

    let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();

    let reg = mci0.reg().unwrap().next().unwrap();

    info!("mci0 reg: {:#x}", reg.address);

    let reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

    let mci0 = MCI::new(reg_base);
    //? 初始化 MCI
    mci0.reset().unwrap_or_else(|e| error!("reset failed: {:?}", e));
    mci0.dump_register();

    info!("card detected {:?}", mci0.card_detected());

    info!("blk size: {:#x}", mci0.blksize());
    let mut data: [u32; 512] = [0; 512];

    let buf = FsdifBuf {
        buf: &mut data,
        buf_dma: 0,
        blkcnt: 1,
        blksz: 512,
    };

    let mut cmd_data = FSdifCmdData {
        cmdidx: 17, //24
        cmdarg: 131072*512,
        flag: CmdFlag::WRITE_DATA | CmdFlag::EXP_DATA | CmdFlag::EXP_RESP,
        data: buf,
        success: false,
        response: [0; 4],
    };

    cmd_data.flag = CmdFlag::READ_DATA | CmdFlag::EXP_DATA | CmdFlag::EXP_RESP;
    let _ = mci0.block_size_set(512);
    let _ = mci0.block_count_set(1);
    let r = mci0.pio_transfer(&cmd_data);
    match r {
        Ok(_) => {
            info!("pio_transfer success");
        }
        Err(e) => {
            info!("pio_transfer failed: {:?}", e);
        }
    }
    let r = mci0.poll_wait_pio_end(&mut cmd_data);
    match r {
        Ok(_) => {
            info!("pio_transfer success");
        }
        Err(e) => {
            info!("pio_transfer failed: {:?}", e);
        }
    }
    mci0.dump_register();
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
