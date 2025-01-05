use std::fmt;

use rustyline::error::ReadlineError;

use crate::memory::AllocError;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    OutOfMemory,
    BadAllocationRequest,
    LexerError(String),
    ParseError(String),
    MutableBorrowError,
    BoundsError,
    EvalError(String),
    UnhashableError,
    KeyError,
    IOError(String),
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

    pub fn error_kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Given the relevant source code string, show the error in context
    pub fn print_with_source(&self, source: &str) {
        if let Some(ref pos) = self.pos {
            let mut iter = source.lines().enumerate();

            while let Some((count, line)) = iter.next() {
                // count starts at 0, line numbers start at 1
                if count + 1 == pos.line as usize {
                    println!("error: {}", self);
                    println!("{:5}|{}", pos.line, line);
                    println!("{:5}|{:width$}^", " ", " ", width = pos.column as usize);
                    println!("{:5}|", " ");
                    return;
                }
            }
        } else {
            println!("error: {}", self);
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::IOError(ref reason) => write!(f, "IO Error: {}", reason),
            ErrorKind::LexerError(ref reason) => write!(f, "Parse error: {}", reason),
            ErrorKind::ParseError(ref reason) => write!(f, "Parse error: {}", reason),
            ErrorKind::EvalError(ref reason) => write!(f, "Evaluation error: {}", reason),
            ErrorKind::OutOfMemory => write!(f, "Out of memory!"),
            ErrorKind::BadAllocationRequest => {
                write!(f, "An invalid memory size allocation was requested!")
            }
            ErrorKind::BoundsError => write!(f, "Indexing bounds error"),
            ErrorKind::KeyError => write!(f, "Key does not exist in Dict"),
            ErrorKind::UnhashableError => write!(f, "Attempt to access Dict with unhashable key"),
            ErrorKind::MutableBorrowError => write!(
                f,
                "Attempt to modify a container that is already mutably borrowed"
            ),
        }
    }
}

/// Convert from ReadlineError
impl From<ReadlineError> for RuntimeError {
    fn from(other: ReadlineError) -> RuntimeError {
        RuntimeError::new(ErrorKind::IOError(format!("{}", other)))
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

/// Convenience shorthand function for building a parser error
pub fn err_parser(reason: &str) -> RuntimeError {
    RuntimeError::new(ErrorKind::ParseError(String::from(reason)))
}

/// Convenience shorthand function for building a parser error including a source position
pub fn err_parser_wpos(pos: SourcePos, reason: &str) -> RuntimeError {
    RuntimeError::with_pos(ErrorKind::ParseError(String::from(reason)), pos)
}

/// Convenience shorthand function for building an evaluation error
pub fn err_eval(reason: &str) -> RuntimeError {
    RuntimeError::new(ErrorKind::EvalError(String::from(reason)))
}
