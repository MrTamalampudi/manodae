use std::{fmt::Debug, hash::Hash};

use indexmap::IndexSet;

use crate::production::Production;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Symbol {
    TERMINAL(String),
    NONTERMINAL(String),
}

pub fn unique_symbols<AST, Token, TranslatorStack>(
    productions: &Vec<Production<AST, Token, TranslatorStack>>,
) -> IndexSet<Symbol> {
    let mut symbols: IndexSet<Symbol> = IndexSet::new();
    for production in productions.iter() {
        for symbol in production.body.iter() {
            symbols.insert(symbol.clone());
        }
    }
    //eofff
    symbols.insert(Symbol::TERMINAL(String::from("EOF")));
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

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match self {
            Symbol::NONTERMINAL(string) | Self::TERMINAL(string) => string.clone(),
        }
    }
}
