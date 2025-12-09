use std::{fmt::Debug, hash::Hash};

use indexmap::IndexMap;

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

pub struct Symbols {
    map: IndexMap<Symbol, usize>,
    vec: Vec<Symbol>,
    terminals: Vec<usize>,
    non_terminals: Vec<usize>,
}

impl Symbols {
    pub fn intern(&mut self, symbol: Symbol) -> usize {
        if let Some(&id) = self.map.get(&symbol) {
            return id;
        }
        let id = self.map.len();
        self.map.insert(symbol.clone(), id);
        self.vec.push(symbol);

        id
    }

    #[must_use]
    pub fn lookup(&self, id: usize) -> Symbol {
        self.vec[id].clone()
    }

    #[must_use]
    pub fn reverse_lookup(&self, symbol: &Symbol) -> Option<usize> {
        self.map.get(symbol).map(|x| *x)
    }

    #[inline]
    /// returns true if the id is terminal else false
    pub fn terminal(&self, id: &usize) -> bool {
        self.terminals.contains(&id)
    }
    #[inline]
    /// returns true if the id is non_terminal else false
    pub fn non_terminal(&self, id: &usize) -> bool {
        self.non_terminals.contains(&id)
    }
}
