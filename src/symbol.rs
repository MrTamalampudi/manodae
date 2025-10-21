use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Symbol {
    TERMINAL(String),
    NONTERMINAL(String),
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

impl From<&Symbol> for String {
    fn from(value: &Symbol) -> Self {
        value.to_string()
    }
}
