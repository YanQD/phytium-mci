#[derive(Debug)]
pub enum FsdifError {
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

pub type FsdifResult<T=()> = Result<T, FsdifError>;