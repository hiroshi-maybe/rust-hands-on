use crate::memory::AllocError;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    OutOfMemory,
    BadAllocationRequest,
}

/// Source code position
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourcePos {
    pub line: u32,
    pub column: u32,
}

/// An Eval-rs runtime error type
#[derive(Debug, PartialEq)]
pub struct RuntimeError {
    kind: ErrorKind,
    pos: Option<SourcePos>,
}

impl RuntimeError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, pos: None }
    }
}

/// Convert from AllocError
impl From<AllocError> for RuntimeError {
    fn from(other: AllocError) -> RuntimeError {
        match other {
            AllocError::OOM => RuntimeError::new(ErrorKind::OutOfMemory),
            AllocError::BadRequest => RuntimeError::new(ErrorKind::BadAllocationRequest),
        }
    }
}
