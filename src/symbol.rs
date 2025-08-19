use std::{collections::HashSet, fmt::Debug};

use crate::production_LR1::Production_LR1;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    TERMINAL(String),
    NONTERMINAL(String),
    NONE,
}

pub fn unique_symbols<AST, Token, TranslatorStack>(
    productions: &Vec<&Production_LR1<AST, Token, TranslatorStack>>,
) -> HashSet<Symbol> {
    let mut symbols: HashSet<Symbol> = HashSet::new();
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
            _ => String::from("None&&"),
        }
    }
}
