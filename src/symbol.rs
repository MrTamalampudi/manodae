use std::fmt::Debug;

use crate::{production::Production, terminal::Terminal};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol<T>
where
    T: PartialEq + Debug + Clone + Eq + Terminal<T>,
{
    TERMINAL(T),
    NONTERMINAL(String),
    NONE,
}

pub fn unique_symbols<T: PartialEq + Debug + Clone + Eq + Terminal<T>>(
    productions: &Vec<Production<T>>,
) -> Vec<Symbol<T>> {
    let mut symbols: Vec<Symbol<T>> = Vec::new();

    for production in productions.iter() {
        for symbol in production.body.iter() {
            if !symbols.contains(symbol) {
                symbols.push(symbol.clone());
            }
        }
    }

    symbols.push(Symbol::TERMINAL(T::get_ending_token()));
    symbols
}

impl<T: Debug + Clone + Eq + Terminal<T>> Symbol<T> {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::TERMINAL(_))
    }

    pub fn is_non_terminal(&self) -> bool {
        matches!(self, Symbol::NONTERMINAL(_))
    }
}
