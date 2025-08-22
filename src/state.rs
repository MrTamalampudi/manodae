use std::{collections::HashMap, fmt::Debug};

use crate::{action::Action, conflict::ConflictType, item::Items, production::Production};

#[derive(Debug, Clone)]
pub struct State<'a, AST, Token, TranslatorStack> {
    pub state: usize,
    pub items: Items<'a, AST, Token, TranslatorStack>,
    pub transition_productions: Vec<&'a Production<AST, Token, TranslatorStack>>,
    pub action: HashMap<String, Action>,
    pub goto: HashMap<String, usize>,
    pub conflicts: HashMap<String, ConflictType>,
}
