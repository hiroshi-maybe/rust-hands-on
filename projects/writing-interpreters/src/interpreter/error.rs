use crate::memory::AllocError;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    OutOfMemory,
    BadAllocationRequest,
    LexerError(String),
}

/// Source code position
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SourcePos {
    pub line: u32,
    pub column: u32,
}

impl SourcePos {
    fn new(line: u32, column: u32) -> SourcePos {
        SourcePos { line, column }
    }
}

/// Convenience shorthand function for building a SourcePos
pub fn spos(line: u32, column: u32) -> SourcePos {
    SourcePos::new(line, column)
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

    pub fn with_pos(kind: ErrorKind, pos: SourcePos) -> RuntimeError {
        RuntimeError {
            kind: kind,
            pos: Some(pos),
        }
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

/// Convenience shorthand function for building a lexer error
pub fn err_lexer(pos: SourcePos, reason: &str) -> RuntimeError {
    RuntimeError::with_pos(ErrorKind::LexerError(String::from(reason)), pos)
}
