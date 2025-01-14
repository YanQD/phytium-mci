/** @name Register Map
 *
 * Register offsets from the base address of an SD device.
 * @{
 */
pub const FSDIF_CNTRL_OFFSET: u32 = 0x00; // the controller config reg
pub const FSDIF_PWREN_OFFSET: u32 = 0x04; // the power enable reg
pub const FSDIF_CLKDIV_OFFSET: u32 = 0x08; // the clock divider reg
pub const FSDIF_CLKENA_OFFSET: u32 = 0x10; // the clock enable reg
pub const FSDIF_TMOUT_OFFSET: u32 = 0x14; // the timeout reg
pub const FSDIF_CTYPE_OFFSET: u32 = 0x18; // the card type reg
pub const FSDIF_BLK_SIZ_OFFSET: u32 = 0x1C; // the block size reg
pub const FSDIF_BYT_CNT_OFFSET: u32 = 0x20; // the byte count reg
pub const FSDIF_INT_MASK_OFFSET: u32 = 0x24; // the interrupt mask reg
pub const FSDIF_CMD_ARG_OFFSET: u32 = 0x28; // the command argument reg
pub const FSDIF_CMD_OFFSET: u32 = 0x2C; // the command reg
pub const FSDIF_RESP0_OFFSET: u32 = 0x30; // the response reg0
pub const FSDIF_RESP1_OFFSET: u32 = 0x34; // the response reg1
pub const FSDIF_RESP2_OFFSET: u32 = 0x38; // the response reg2
pub const FSDIF_RESP3_OFFSET: u32 = 0x3C; // the response reg3
pub const FSDIF_MASKED_INTS_OFFSET: u32 = 0x40; // the masked interrupt status reg
pub const FSDIF_RAW_INTS_OFFSET: u32 = 0x44; // the raw interrupt status reg
pub const FSDIF_STATUS_OFFSET: u32 = 0x48; // the status reg
pub const FSDIF_FIFOTH_OFFSET: u32 = 0x4C; // the FIFO threshold watermark reg
pub const FSDIF_CARD_DETECT_OFFSET: u32 = 0x50; // the card detect reg
pub const FSDIF_CARD_WRTPRT_OFFSET: u32 = 0x54; // the card write protect reg
pub const FSDIF_CKSTS_OFFSET: u32 = 0x58; // the ciu ready
pub const FSDIF_TRAN_CARD_CNT_OFFSET: u32 = 0x5C; // the transferred CIU card byte count reg
pub const FSDIF_TRAN_FIFO_CNT_OFFSET: u32 = 0x60; // the transferred host to FIFO byte count reg
pub const FSDIF_DEBNCE_OFFSET: u32 = 0x64; // the debounce count reg
pub const FSDIF_UID_OFFSET: u32 = 0x68; // the user ID reg
pub const FSDIF_VID_OFFSET: u32 = 0x6C; // the controller version ID reg
pub const FSDIF_HWCONF_OFFSET: u32 = 0x70; // the hardware configuration reg
pub const FSDIF_UHS_REG_OFFSET: u32 = 0x74; // the UHS-I reg
pub const FSDIF_CARD_RESET_OFFSET: u32 = 0x78; // the card reset reg
pub const FSDIF_BUS_MODE_OFFSET: u32 = 0x80; // the bus mode reg
pub const FSDIF_DESC_LIST_ADDRL_OFFSET: u32 = 0x88; // the descriptor list low base address reg
pub const FSDIF_DESC_LIST_ADDRH_OFFSET: u32 = 0x8C; // the descriptor list high base address reg
pub const FSDIF_DMAC_STATUS_OFFSET: u32 = 0x90; // the internal DMAC status reg
pub const FSDIF_DMAC_INT_EN_OFFSET: u32 = 0x94; // the internal DMAC interrupt enable reg
pub const FSDIF_CUR_DESC_ADDRL_OFFSET: u32 = 0x98; // the current host descriptor low address reg
pub const FSDIF_CUR_DESC_ADDRH_OFFSET: u32 = 0x9C; // the current host descriptor high address reg
pub const FSDIF_CUR_BUF_ADDRL_OFFSET: u32 = 0xA0; // the current buffer low address reg
pub const FSDIF_CUR_BUF_ADDRH_OFFSET: u32 = 0xA4; // the current buffer high address reg
pub const FSDIF_CARD_THRCTL_OFFSET: u32 = 0x100; // the card threshold control reg
pub const FSDIF_CLK_SRC_OFFSET: u32 = 0x108; // the UHS register extension
pub const FSDIF_EMMC_DDR_REG_OFFSET: u32 = 0x10C; // the EMMC DDR reg
pub const FSDIF_ENABLE_SHIFT_OFFSET: u32 = 0x110; // the enable phase shift reg
pub const FSDIF_DATA_OFFSET: u32 = 0x200; // the data FIFO access

pub const FSDIF_TIMEOUT:u32 = 50000; /* timeout for retries */
pub const FSDIF_DELAY_US:u32 = 5;
pub const FSDIF_MAX_FIFO_CNT:u32 = 0x800;
