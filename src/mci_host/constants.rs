use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostCmdType {
    Normal = 0,  // 普通命令
    Suspend = 1, // 挂起命令
    Resume = 2,  // 恢复命令
    Abort = 3,   // 中止命令
    Empty = 4,   // 空命令
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostResponseType {
    None = 0, // 无响应
    R1 = 1,   // 响应类型: R1
    R1b = 2,  // 响应类型: R1b
    R2 = 3,   // 响应类型: R2
    R3 = 4,   // 响应类型: R3
    R4 = 5,   // 响应类型: R4
    R5 = 6,   // 响应类型: R5
    R5b = 7,  // 响应类型: R5b
    R6 = 8,   // 响应类型: R6s
    R7 = 9,   // 响应类型: R7
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostVoltage {
    NotSet = 0,    // 表示当前电压设置未由用户设置
    Volts3V3 = 1,  // 卡操作电压约为 3.3V
    Volts3V0 = 2,  // 卡操作电压约为 3.0V
    Volts1V8 = 3,  // 卡操作电压约为 1.8V
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostDataPacketFormat {
    MSBFirst = 0, // 数据包格式: MSB 优先
    LSBFirst = 1, // 数据包格式: LSB 优先
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostBusWdith {
    Bit1 = 0,
    Bit4 = 1,
    Bit8 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostCommonCommand {
    GoIdleState = 0,         // Go Idle State
    AllSendCid = 2,          // All Send CID
    SetDsr = 4,              // Set DSR
    SelectCard = 7,          // Select Card
    SendCsd = 9,             // Send CSD
    SendCid = 10,            // Send CID
    StopTransmission = 12,   // Stop Transmission
    SendStatus = 13,         // Send Status
    GoInactiveState = 15,    // Go Inactive State
    SetBlockLength = 16,     // Set Block Length
    ReadSingleBlock = 17,    // Read Single Block
    ReadMultipleBlock = 18,  // Read Multiple Block
    SetBlockCount = 23,      // Set Block Count
    WriteSingleBlock = 24,   // Write Single Block
    WriteMultipleBlock = 25, // Write Multiple Block
    ProgramCsd = 27,         // Program CSD
    SetWriteProtect = 28,    // Set Write Protect
    ClearWriteProtect = 29,  // Clear Write Protect
    SendWriteProtect = 30,   // Send Write Protect
    Erase = 38,              // Erase
    LockUnlock = 42,         // Lock Unlock
    ApplicationCommand = 55, // Send Application Command
    GeneralCommand = 56,     // General Purpose Command
    ReadOcr = 58,            // Read OCR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCISDIOCommand {
    SendRelativeAddress = 3, // Send Relative Address
    SendOperationCondition = 5, // Send Operation Condition
    SendInterfaceCondition = 8, // Send Interface Condition
    RWIODirect = 52, // Read/Write I/O Direct
    RWIODirectExtended = 53, // Read/Write I/O Direct Extended
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCISDIOCCCRAddr {
    SDIOVer = 0x00,                 // CCCR & SDIO version
    SDVersion = 0x01,               // SD version
    IOEnable = 0x02,                // io enable register
    IOReady = 0x03,                 // io ready register
    IOIntEnable = 0x04,             // io interrupt enable register
    IOIntPending = 0x05,            // io interrupt pending register
    IOAbort = 0x06,                 // io abort register
    BusInterface = 0x07,            // bus interface register
    CardCapability = 0x08,          // card capability register
    CommonCISPointer = 0x09,        // common CIS pointer register
    BusSuspend = 0x0C,              // bus suspend register
    FunctionSelect = 0x0D,          // function select register
    ExecutionFlag = 0x0E,           // execution flag register
    ReadyFlag = 0x0F,               // ready flag register
    FN0BlockSizeLow = 0x10,         // FN0 block size register
    FN0BlockSizeHigh = 0x11,        // FN0 block size register
    PowerControl = 0x12,            // power control register
    BusSpeed = 0x13,                // bus speed register
    UHSITimingSupport = 0x14,       // UHS-I timing support register
    DriverStrength = 0x15,          // Driver strength register
    InterruptExtension = 0x16,      // Interrupt extension register
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostSDCommand {
    SendRelativeAddress = 3,      // Send Relative Address
    Switch = 6,                   // Switch Function
    SendInterfaceCondition = 8,   // Send Interface Condition
    VoltageSwitch = 11,           // Voltage Switch
    SpeedClassControl = 20,       // Speed Class control
    SendTuningBlock = 19,         // Send Tuning Block
    EraseWriteBlockStart = 32,    // Write Block Start
    EraseWriteBlockEnd = 33,      // Write Block End
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MCIHostCardStatusFlag: u32 {
        const OUT_OF_RANGE                  = 1 << 31; // Out of range status bit
        const ADDRESS_ERROR                 = 1 << 30; // Address error status bit
        const BLOCK_LENGTH_ERROR            = 1 << 29; // Block length error status bit
        const ERASE_SEQUENCE_ERROR          = 1 << 28; // Erase sequence error status bit
        const ERASE_PARAMETER_ERROR         = 1 << 27; // Erase parameter error status bit
        const WRITE_PROTECT_VIOLATION       = 1 << 26; // Write protection violation status bit
        const CARD_IS_LOCKED                = 1 << 25; // Card locked status bit
        const LOCK_UNLOCK_FAILED            = 1 << 24; // Lock/unlock error status bit
        const COMMAND_CRC_ERROR             = 1 << 23; // CRC error status bit
        const ILLEGAL_COMMAND               = 1 << 22; // Illegal command status bit
        const CARD_ECC_FAILED               = 1 << 21; // Card ECC error status bit
        const CARD_CONTROLLER_ERROR         = 1 << 20; // Internal card controller error status bit
        const ERROR                         = 1 << 19; // A general or an unknown error status bit
        const CID_CSD_OVERWRITE             = 1 << 16; // CID/CSD overwrite status bit
        const WRITE_PROTECT_ERASE_SKIP      = 1 << 15; // Write protection erase skip status bit
        const CARD_ECC_DISABLED             = 1 << 14; // Card ECC disabled status bit
        const ERASE_RESET                   = 1 << 13; // Erase reset status bit
        const READY_FOR_DATA                = 1 << 8;  // Ready for data status bit
        const SWITCH_ERROR                  = 1 << 7;  // Switch error status bit
        const APPLICATION_COMMAND           = 1 << 5;  // Application command enabled status bit
        const AUTHENTICATION_SEQUENCE_ERROR = 1 << 3;  // Error in the sequence of authentication process
        const ALL_ERROR_FLAG = 0xFFF0008;    // All error status bits
    }
}

