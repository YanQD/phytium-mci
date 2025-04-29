#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::time::Duration;

    use alloc::vec::Vec;
    use bare_test::{
        globals::{global_val, PlatformInfoKind}, mem::mmu::iomap, 
        time::spin_delay
    };
    use log::*;
    use phytium_mci::{iopad::PAD_ADDRESS, sd::SdCard, *};
    #[test]
    fn test_work() {
        let fdt = match &global_val().platform_info {
            PlatformInfoKind::DeviceTree(fdt) => fdt.get(),
            // _ => panic!("unsupported platform"),
        };
    
        let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();
    
        let reg = mci0.reg().unwrap().next().unwrap();
    
        info!("mci0 reg: {:#x},mci0 reg size: {:#x}", reg.address, reg.size.unwrap());
    
        let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());
    
        let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);
    
        let iopad = IoPad::new(iopad_reg_base);
    
        let mut sdcard = SdCard::example_instance(mci_reg_base,iopad);

        ////////////////////// SD card init finished //////////////////////

        let mut buffer: Vec<u32> = Vec::with_capacity(512);
        buffer.resize(512, 0);
        for i in 0..buffer.len() {
            buffer[i] = i as u32;
        }

        let _ = sdcard.write_blocks(&mut buffer, 131072 + 200, 1);

        let mut receive_buf = Vec::new();
        
        let _ = sdcard.read_blocks(&mut receive_buf, 131072+200,1);
    
        for i in 0..receive_buf.len() {
            warn!("{:x},{:x},{:x},{:x}",
            receive_buf[i] as u8,
            (receive_buf[i] >> 8) as u8,
            (receive_buf[i] >> 16) as u8,
            (receive_buf[i] >> 24) as u8);
        }
        error!("test_work passed\n");
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