use super::{error::SourcePos, RuntimeError};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    OpenParen,
    CloseParen,
    Symbol(String),
    Dot,
    // Text(String),
    // Quote,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub pos: SourcePos,
    pub token: TokenType,
}

// tokenize a String
pub fn tokenize(input: &str) -> Result<Vec<Token>, RuntimeError> {
    unimplemented!()
}
