use log::*;
use super::mci_data::MCIData;
use super::MCI;
use super::err::*;
use super::constants::*;
use super::regs::*;

impl MCI {

    pub(crate) fn pio_write_data<'a>(&self, data: &mut MCIData) -> MCIResult {
        let reg = self.config.reg();
        let wr_times: usize = (data.datalen() / 4) as usize; /* u8 --> u32 */
        let buf = data.buf_mut();

        /* write fifo data */
        reg.write_reg(MCICmd::DAT_WRITE);
        for i in 0..wr_times {
            reg.write_reg(FsdifData::from_bits_truncate(buf[i]));
        }
        Ok(())
    }

    pub(crate) fn pio_read_data(&self, data: &mut MCIData) -> MCIResult {
        let reg = self.config.reg();
        let datalen = data.datalen();
        let rd_times = (datalen / 4) as usize; /* u8 --> u32 */
        let buf = data.buf_mut();

        if datalen > MCI_MAX_FIFO_CNT {
            error!("Fifo do not support writing more than 0x{:x}.",MCI_MAX_FIFO_CNT);
            return Err(MCIError::NotSupport);
        }
        for i in 0..rd_times {
            buf[i] = reg.read_reg::<FsdifData>().bits();
        }
        Ok(())
    }
    
}