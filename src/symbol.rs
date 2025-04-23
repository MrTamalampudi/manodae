use crate::{production::Production, TokenType};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    TERMINAL(TokenType),
    NONTERMINAL(String),
    NONE,
}

pub fn unique_symbols(productions: &Vec<Production>) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for production in productions.iter() {
        for symbol in production.body.iter() {
            if !symbols.contains(symbol) {
                symbols.push(symbol.clone());
            }
        }
    }

    symbols.push(Symbol::TERMINAL(TokenType::EOF));
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
