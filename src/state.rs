use std::{collections::HashMap, fmt::Debug};

use crate::{action::Action, conflict::ConflictType, production::Production, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct State<T, TokenType> {
    pub state: usize,
    pub productions: Vec<Production<T, TokenType>>,
    pub transition_symbol: Symbol,
    pub transition_productions: Vec<Production<T, TokenType>>,
    pub action: HashMap<String, Action>,
    pub goto: HashMap<String, usize>,
    pub conflicts: HashMap<String, ConflictType>,
}
