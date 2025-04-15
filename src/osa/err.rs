pub enum FMempError {
    InvalidBuf,
    InitTlsfError,
    BadMalloc,
}

pub type FMempStatus<T=()> = Result<T, FMempError>;