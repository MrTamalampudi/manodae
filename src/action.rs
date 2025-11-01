use std::rc::Rc;

use crate::{production::Production, state::State};

#[derive(Debug, Clone, PartialEq)]
pub enum Action<AST, Token, TranslatorStack> {
    SHIFT(Rc<State<AST, Token, TranslatorStack>>),
    REDUCE(Rc<Production<AST, Token, TranslatorStack>>),
    ACCEPT,
    ERROR(String),
}
