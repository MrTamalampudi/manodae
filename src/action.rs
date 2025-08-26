use crate::{production::Production, state::State};

#[derive(Debug, Clone, PartialEq)]
pub enum Action<'a, AST, Token, TranslatorStack> {
    SHIFT(State<'a, AST, Token, TranslatorStack>),
    REDUCE(&'a Production<AST, Token, TranslatorStack>),
    ACCEPT,
    ERROR(String),
}
