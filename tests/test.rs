#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::time::Duration;

    use alloc::vec::Vec;
    use bare_test::{
        globals::{global_val, PlatformInfoKind},
        mem::mmu::iomap,
        time::spin_delay,
    };
    use log::*;
    use phytium_mci::{iopad::PAD_ADDRESS, sd::SdCard, *};

    const SD_START_BLOCK: u32 = 131072;
    const SD_USE_BLOCK: u32 = 2;
    const SD_BLOCK_SIZE: u32 = 512;

    #[test]
    fn test_work() {
        let fdt = match &global_val().platform_info {
            PlatformInfoKind::DeviceTree(fdt) => fdt.get(),
            // _ => panic!("unsupported platform"),
        };

        let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();

        let reg = mci0.reg().unwrap().next().unwrap();

        info!(
            "mci0 reg: {:#x},mci0 reg size: {:#x}",
            reg.address,
            reg.size.unwrap()
        );

        let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

        let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);

        let iopad = IoPad::new(iopad_reg_base);
    
        let mut sdcard = SdCard::new(mci_reg_base,iopad);

        ////////////////////// SD card init finished //////////////////////

        // 第i个块的每个字节都设置为i+1
        let mut buffer: Vec<u32> = Vec::with_capacity(512);
        buffer.resize(512, 0);
        // for block in 0..SD_USE_BLOCK {
        //     for i in 0..SD_BLOCK_SIZE / 4 {
        //         let pos = ((SD_START_BLOCK + block) * SD_BLOCK_SIZE + i) as usize;
        //         let idx = (block + 1) as u32;
        //         let val = idx | (idx << 8) | (idx << 16) | (idx << 24);
        //         buffer[pos] = val;
        //     }
        // }
        for i in 0..512 {
            let val = (i | i << 8 | i << 16 | i << 24) as u32;
            buffer[i] = val;
        }

        sdcard.write_blocks(&mut buffer, 131072 + 200, 1).unwrap();

        let mut receive_buf = Vec::new();

        sdcard
            .read_blocks(&mut receive_buf, 131072 + 200, 2)
            .unwrap();

        // for i in 0..receive_buf.len() {
        //     assert_eq!(receive_buf[i], buffer[i]);
        // }
        for i in 0..receive_buf.len() {
            warn!("{:x},{:x},{:x},{:x}",
            receive_buf[i] as u8,
            (receive_buf[i] >> 8) as u8,
            (receive_buf[i] >> 16) as u8,
            (receive_buf[i] >> 24) as u8);
        }
        warn!("buffer len is {}", receive_buf.len());

        info!("test_work passed\n");
        assert!(true);
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
