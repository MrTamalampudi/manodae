use std::fmt::Debug;

use crate::production::Production;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    TERMINAL(String),
    NONTERMINAL(String),
    NONE,
}

pub fn unique_symbols<T>(productions: &Vec<Production<T>>) -> Vec<Symbol> {
    let mut symbols: Vec<Symbol> = Vec::new();

    for production in productions.iter() {
        for symbol in production.body.iter() {
            if !symbols.contains(symbol) {
                symbols.push(symbol.clone());
            }
        }
    }
    //eofff
    symbols.push(Symbol::TERMINAL(String::from("EOF")));
    symbols
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::TERMINAL(_))
    }

    pub fn is_non_terminal(&self) -> bool {
        matches!(self, Symbol::NONTERMINAL(_))
    }
}
