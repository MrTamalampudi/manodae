use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use indexmap::IndexMap;

use crate::interner::Interner;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub usize);

impl Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SymbolId {}", self.0))
    }
}

#[derive(Debug, Clone)]
pub struct Symbols {
    pub map: IndexMap<Symbol, SymbolId>,
    pub vec: Vec<Symbol>,
    pub terminals: Vec<SymbolId>,
    pub non_terminals: Vec<SymbolId>,
}

pub(crate) const AUGMENT_START_SYMBOL_ID: SymbolId = SymbolId(0);
pub(crate) const EOF_SYMBOL_ID: SymbolId = SymbolId(1);
pub(crate) const START_SYMBOL_ID: SymbolId = SymbolId(2);

impl Interner for Symbols {
    type T = Symbol;
    type Id = SymbolId;
    fn intern(&mut self, symbol: Symbol) -> SymbolId {
        if let Some(&id) = self.map.get(&symbol) {
            return id;
        }
        let id = SymbolId(self.map.len());
        match symbol {
            Symbol::NONTERMINAL(_) => self.non_terminals.push(id),
            Symbol::TERMINAL(_) => self.terminals.push(id),
        }
        self.map.insert(symbol.clone(), id);
        self.vec.push(symbol);

        id
    }

    fn lookup(&self, id: SymbolId) -> Symbol {
        self.vec[id.0].clone()
    }

    fn reverse_lookup(&self, symbol: &Symbol) -> Option<SymbolId> {
        self.map.get(symbol).map(|x| *x)
    }
}

impl Symbols {
    pub fn new() -> Symbols {
        let mut symbols = Symbols {
            map: IndexMap::new(),
            vec: vec![],
            terminals: vec![],
            non_terminals: vec![],
        };
        symbols.intern(Symbol::NONTERMINAL(String::from("S'")));
        symbols.intern(Symbol::TERMINAL(String::from("EOF")));
        symbols.intern(Symbol::NONTERMINAL("Start".to_string()));
        symbols
    }
    #[inline]
    /// returns true if the id is terminal else false
    pub fn terminal(&self, id: &SymbolId) -> bool {
        self.terminals.contains(id)
    }
    #[inline]
    /// returns true if the id is non_terminal else false
    pub fn non_terminal(&self, id: &SymbolId) -> bool {
        self.non_terminals.contains(id)
    }
}
