use core::panic;
use std::{iter::Peekable, marker::PhantomData};

use crate::interpreter::error::{err_parser, err_parser_wpos};

use super::{
    error::SourcePos,
    lexer::{tokenize, Token, TokenType},
    pair::Pair,
    safeptr::{MutatorScope, TaggedCellPtr, TaggedScopedPtr},
    taggedptr::Value,
    MutatorView, RuntimeError,
};

/// Parse the given string into an AST
pub fn parse<'guard>(
    mem: &'guard MutatorView,
    input: &str,
) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
    parse_tokens(mem, tokenize(input)?)
}

fn parse_tokens<'guard>(
    mem: &'guard MutatorView,
    tokens: Vec<Token>,
) -> Result<TaggedScopedPtr<'guard>, RuntimeError> {
    let mut peekable = tokens.iter().peekable();
    parse_sexpr(mem, &mut peekable)
}

//
// Parse a single s-expression
//
// Must be a
//  * symbol
//  * or a list
//
fn parse_sexpr<'guard, 'i, I: 'i>(
    mem: &'guard MutatorView,
    tokens: &mut Peekable<I>,
) -> Result<TaggedScopedPtr<'guard>, RuntimeError>
where
    I: Iterator<Item = &'i Token>,
{
    use self::TokenType::*;

    match tokens.peek() {
        // '('
        Some(&&Token {
            token: OpenParen,
            pos: _,
        }) => {
            tokens.next();
            parse_list(mem, tokens)
        }
        // Symbol
        Some(&&Token {
            token: Symbol(ref name),
            pos: _,
        }) => {
            tokens.next();
            // the symbol 'nil' is reinterpreted as a literal nil value
            if name == "nil" {
                Ok(mem.nil())
            } else {
                Ok(mem.lookup_sym(name))
            }
        }
        None => {
            tokens.next();
            Ok(mem.nil())
        }
        // '.'
        Some(&&Token { token: Dot, pos }) => Err(err_parser_wpos(pos, "Invalid symbol '.'")),
        // ')'
        Some(&&Token {
            token: CloseParen,
            pos,
        }) => Err(err_parser_wpos(pos, "Unmatched close parenthesis")),
    }
}

//
// A list is either
// * empty
// * a sequence of s-expressions
//
// If the first list token is:
//  * a CloseParen, it's a Nil value
//  * a Dot, this is illegal
//
// If a list token is:
//  * a Dot, it must be followed by an s-expression and a CloseParen
//
fn parse_list<'guard, 'i, I: 'i>(
    mem: &'guard MutatorView,
    tokens: &mut Peekable<I>,
) -> Result<TaggedScopedPtr<'guard>, RuntimeError>
where
    I: Iterator<Item = &'i Token>,
{
    use self::TokenType::*;

    match tokens.peek() {
        // ')'
        Some(&&Token {
            token: CloseParen,
            pos: _,
        }) => {
            tokens.next();
            return Ok(mem.nil());
        }
        // '.'
        Some(&&Token { token: Dot, pos }) => {
            return Err(err_parser_wpos(
                pos,
                "Unexpected '.' dot after open-parenthesis",
            ));
        }
        _ => (),
    }

    let mut list = PairList::open(mem);
    loop {
        match tokens.peek() {
            // '(..' - Nested list
            Some(&&Token {
                token: OpenParen,
                pos,
            }) => {
                tokens.next();
                list.push(mem, parse_list(mem, tokens)?, pos)?;
            }
            // Symbol
            Some(&&Token {
                token: Symbol(_),
                pos,
            }) => {
                list.push(mem, parse_sexpr(mem, tokens)?, pos)?;
            }
            // ')' - End of the current list
            Some(&&Token {
                token: CloseParen,
                pos: _,
            }) => {
                tokens.next();
                break;
            }
            // ". S-exp)" - Dotted pair ending with ')'
            Some(&&Token { token: Dot, pos }) => {
                tokens.next();
                list.dot(mem, parse_sexpr(mem, tokens)?, pos);

                // the only valid sequence here on out is Dot s-expression CloseParen
                match tokens.peek() {
                    Some(&&Token {
                        token: CloseParen,
                        pos: _,
                    }) => (),

                    Some(&&Token { token: _, pos }) => {
                        return Err(err_parser_wpos(
                            pos,
                            "Dotted pair must be closed by a ')' close-parenthesis",
                        ));
                    }

                    None => return Err(err_parser("Unexpected end of code stream")),
                }
            }
            None => {
                return Err(err_parser("Unexpected end of code stream"));
            }
        }
    }

    Ok(list.close(mem))
}

// A linked list, internal to the parser to simplify the code and is stored on the Rust stack
struct PairList<'guard> {
    head: TaggedCellPtr,
    tail: TaggedCellPtr,
    _guard: PhantomData<&'guard dyn MutatorScope>,
}

impl<'guard> PairList<'guard> {
    /// Create a new empty list
    fn open(_guard: &'guard dyn MutatorScope) -> PairList {
        PairList {
            head: TaggedCellPtr::new_nil(),
            tail: TaggedCellPtr::new_nil(),
            _guard: PhantomData,
        }
    }

    /// Consume the list and return the pair at the head
    fn close(self, guard: &'guard dyn MutatorScope) -> TaggedScopedPtr<'guard> {
        self.head.get(guard)
    }

    /// Move the given value to managed memory and append it to the list
    fn push(
        &mut self,
        mem: &'guard MutatorView,
        value: TaggedScopedPtr<'guard>,
        pos: SourcePos,
    ) -> Result<(), RuntimeError> {
        if let Value::Pair(old_tail) = *self.tail.get(mem) {
            let new_tail = old_tail.append(mem, value)?;
            self.tail.set(new_tail);

            old_tail.set_second_source_code_pos(pos);

            if let Value::Pair(new_tail) = *new_tail {
                new_tail.set_first_source_code_pos(pos);
            }
        } else {
            // Initially pushed
            let pair = Pair::new();
            pair.first.set(value);

            self.head.set(mem.alloc_tagged(pair)?);
            self.tail.copy_from(&self.head);
        }

        Ok(())
    }

    /// Apply dot-notation to set the second value of the last pair of the list
    fn dot(
        &mut self,
        guard: &'guard dyn MutatorScope,
        value: TaggedScopedPtr<'guard>,
        pos: SourcePos,
    ) {
        if let Value::Pair(pair) = *self.tail.get(guard) {
            pair.dot(value);
            pair.set_second_source_code_pos(pos);
        } else {
            panic!("Cannot dot an empty PairList::tail!")
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::memory::{Memory, Mutator, MutatorView};
    use super::super::printer::print;
    use super::*;

    fn check(input: &str, expect: &str) {
        let mem = Memory::new();

        struct Test<'a> {
            input: &'a str,
            expect: &'a str,
        }

        impl<'a> Mutator for Test<'a> {
            type Input = (); // not convenient to pass &str as Input as Output because of the lifetime
            type Output = ();

            fn run(&self, mem: &MutatorView, _: Self::Input) -> Result<Self::Output, RuntimeError> {
                let ast = parse(mem, self.input)?;
                println!(
                    "expect: {}\ngot:    {}\ndebug:  {:?}",
                    &self.expect, &ast, *ast
                );
                assert_eq!(print(*ast), self.expect);

                Ok(())
            }
        }

        let test = Test { input, expect };
        mem.mutate(&test, ()).unwrap();
    }

    #[test]
    fn parse_empty_list() {
        let input = String::from("()");
        let expect = String::from("nil");
        check(&input, &expect);
    }

    #[test]
    fn parse_nil() {
        let input = String::from("(a . nil)");
        let expect = String::from("(a)");
        check(&input, &expect);
    }

    #[test]
    fn parse_symbol() {
        let input = String::from("a");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_list() {
        let input = String::from("(a)");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_list_nested1() {
        let input = String::from("((a))");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_list_nested2() {
        let input = String::from("(a (b c) d)");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_list_nested3() {
        let input = String::from("(a b (c (d)))");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_longer_list() {
        let input = String::from("(a b c)");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_dot_notation() {
        let input = String::from("(a . b)");
        let expect = input.clone();
        check(&input, &expect);
    }

    #[test]
    fn parse_dot_notation_longer() {
        let input = String::from("((a . b) . (c . d))");
        let expect = String::from("((a . b) c . d)");
        check(&input, &expect);
    }

    #[test]
    fn parse_dot_notation_with_nil() {
        let input = String::from("(a . ())");
        let expect = String::from("(a)");
        check(&input, &expect);
    }
}
