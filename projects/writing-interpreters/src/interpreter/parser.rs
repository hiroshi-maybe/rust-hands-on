use super::{
    lexer::{tokenize, Token},
    safeptr::TaggedScopedPtr,
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
    unimplemented!()
}
