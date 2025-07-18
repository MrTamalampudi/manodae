use std::{collections::HashMap, fmt::Debug};

use crate::{action::Action, conflict::ConflictType, production::Production, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct State<AST, Token, TranslatorStack> {
    pub state: usize,
    pub productions: Vec<Production<AST, Token, TranslatorStack>>,
    pub transition_symbol: Symbol,
    pub transition_productions: Vec<Production<AST, Token, TranslatorStack>>,
    pub action: HashMap<String, Action>,
    pub goto: HashMap<String, usize>,
    pub conflicts: HashMap<String, ConflictType>,
}
