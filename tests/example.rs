    // let mut data: [u32; 512] = [0xA5A5A5A5; 512];
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
    // let r = mci0.pio_transfer(&cmd_data);
    // match r {
    //     Ok(_) => {
    //         info!("pio_transfer success");
    //     }
    //     Err(e) => {
    //         info!("pio_transfer failed: {:?}", e);
    //     }
    // }
    // let r = mci0.poll_wait_pio_end(&mut cmd_data);
    // match r {
    //     Ok(_) => {
    //         info!("pio_transfer success");
    //     }
    //     Err(e) => {
    //         info!("pio_transfer failed: {:?}", e);
    //     }
    // }
    cmd_data.flag = CmdFlag::READ_DATA | CmdFlag::EXP_DATA | CmdFlag::EXP_RESP;
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