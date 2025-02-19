#[derive(Debug)]
pub(crate) struct MCIData<'a> {
    buf: &'a mut [u32],
    buf_dma: u32,
    blksz: u32,
    blkcnt: u32,
    datalen: u32,
}

impl MCIData<'_> {
   pub(crate) fn blksz(&self) -> u32 {
        self.blksz
    }

    pub(crate) fn blkcnt(&self) -> u32 {
        self.blkcnt
    }

    pub(crate) fn datalen(&self) -> u32 {
        self.datalen
    }
    pub(crate) fn buf(&self) -> &[u32] {
        self.buf
    }
    pub(crate) fn buf_mut(&mut self) -> &mut [u32] {
        self.buf
    }
}