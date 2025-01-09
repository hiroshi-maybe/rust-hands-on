use super::{
    error::{err_lexer, spos, SourcePos},
    RuntimeError,
};

// key characters
const OPEN_PAREN: char = '(';
const CLOSE_PAREN: char = ')';
const SPACE: char = ' ';
const TAB: char = '\t';
const CR: char = '\r';
const LF: char = '\n';
const DOT: char = '.';
const DOUBLE_QUOTE: char = '"';
const SINGLE_QUOTE: char = '\'';

#[derive(Debug, PartialEq)]
pub enum TokenType {
    OpenParen,
    CloseParen,
    Symbol(String),
    Dot,
    Number(isize),
    // Text(String),
    // Quote,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub pos: SourcePos,
    pub token: TokenType,
}

impl Token {
    fn new(pos: SourcePos, token: TokenType) -> Token {
        Token { pos, token }
    }
}

// tokenize a String
pub fn tokenize(input: &str) -> Result<Vec<Token>, RuntimeError> {
    let mut tokens = Vec::new();

    // start line numbering at 1, the first character of each line being number 0
    let mut line = 1;
    let mut column = 0;

    let mut chars = input.chars();
    let mut current = chars.next();

    loop {
        match current {
            Some(OPEN_PAREN) => {
                tokens.push(Token::new(spos(line, column), TokenType::OpenParen));
                current = chars.next();
                column += 1;
            }
            Some(CLOSE_PAREN) => {
                tokens.push(Token::new(spos(line, column), TokenType::CloseParen));
                current = chars.next();
                column += 1;
            }
            Some(DOT) => {
                tokens.push(Token::new(spos(line, column), TokenType::Dot));
                current = chars.next();
                column += 1;
            }
            Some(SPACE) => {
                column += 1;
                current = chars.next();
            }
            Some(TAB) => {
                return Err(err_lexer(
                    spos(line, column),
                    "tabs are not valid whitespace",
                ));
            }
            Some(CR) => {
                current = chars.next();
                if let Some(LF) = current {
                    continue;
                }

                line += 1;
                column = 0;
            }
            Some(LF) => {
                line += 1;
                column = 0;
                current = chars.next();
            }
            Some(c) => {
                let symbol_start_column = column;
                let mut symbol = String::new();
                symbol.push(c);
                loop {
                    current = chars.next();
                    match current {
                        Some(c) => {
                            if is_terminating(c) {
                                column += 1;
                                break;
                            } else {
                                symbol.push(c);
                                column += 1;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }

                if let Ok(number) = symbol.parse::<isize>() {
                    tokens.push(Token::new(
                        spos(line, symbol_start_column),
                        TokenType::Number(number),
                    ));
                } else {
                    tokens.push(Token::new(
                        spos(line, symbol_start_column),
                        TokenType::Symbol(symbol),
                    ));
                }
            }
            None => {
                // EOF
                break;
            }
        }
    }

    Ok(tokens)
}

fn is_terminating(c: char) -> bool {
    let terminating = [OPEN_PAREN, CLOSE_PAREN, SPACE, TAB, CR, LF, DOUBLE_QUOTE];
    terminating.iter().any(|t| *t == c)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lexer_empty_string() {
        if let Ok(tokens) = tokenize("") {
            assert!(tokens.len() == 0);
        } else {
            assert!(false, "unexpected error");
        }
    }

    #[test]
    fn lexer_one_line() {
        if let Ok(tokens) = tokenize("(foo bar baz)") {
            assert!(tokens.len() == 5);
            assert_eq!(tokens[0], Token::new(spos(1, 0), TokenType::OpenParen));
            assert_eq!(
                tokens[1],
                Token::new(spos(1, 1), TokenType::Symbol(String::from("foo")))
            );
            assert_eq!(
                tokens[2],
                Token::new(spos(1, 5), TokenType::Symbol(String::from("bar")))
            );
            assert_eq!(
                tokens[3],
                Token::new(spos(1, 9), TokenType::Symbol(String::from("baz")))
            );
            assert_eq!(tokens[4], Token::new(spos(1, 12), TokenType::CloseParen));
        } else {
            assert!(false, "unexpected error");
        }
    }

    #[test]
    fn lexer_multi_line() {
        if let Ok(tokens) = tokenize("( foo\nbar\r\nbaz\n)") {
            assert!(tokens.len() == 5);
            assert_eq!(tokens[0], Token::new(spos(1, 0), TokenType::OpenParen));
            assert_eq!(
                tokens[1],
                Token::new(spos(1, 2), TokenType::Symbol(String::from("foo")))
            );
            assert_eq!(
                tokens[2],
                Token::new(spos(2, 0), TokenType::Symbol(String::from("bar")))
            );
            assert_eq!(
                tokens[3],
                Token::new(spos(3, 0), TokenType::Symbol(String::from("baz")))
            );
            assert_eq!(tokens[4], Token::new(spos(4, 0), TokenType::CloseParen));
        } else {
            assert!(false, "unexpected error");
        }
    }
}
