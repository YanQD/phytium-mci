#[derive(Debug)]
pub enum FsdifError {
    Success,
    Timeout,
    NotInit,
    ShortBuf,
    NotSupport,
    InvalidState,
    TransTimeout,
    CmdTimeout,
    NoCard,
    Busy,
    DmaBufUnalign,
    InvalidTiming,
}
