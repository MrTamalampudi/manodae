#![allow(nonstandard_style)]

pub mod action;
pub mod codegen;
pub mod conflict;
pub mod error;
pub mod first;
pub mod follow;
pub mod grammar;
pub mod interner;
pub mod item;
pub mod parser;
pub mod production;
pub mod render_table;
pub mod state;
pub mod symbol;
pub mod token;

pub mod prelude {
    pub use crate::action::Action;
    pub use crate::action::Action::ACCEPT as A;
    pub use crate::action::Action::ERROR as E;
    pub use crate::action::Action::REDUCE as R;
    pub use crate::action::Action::SHIFT as S;
    pub use crate::codegen::Codegen;
    pub use crate::error::ParseError;
    pub use crate::grammar;
    pub use crate::grammar::Grammar;
    pub use crate::interner::Interner;
    pub use crate::item::Item as I;
    pub use crate::parser::LR1_Parser;
    pub use crate::production::Production;
    pub use crate::production::Production as P;
    pub use crate::production::ProductionId as p;
    pub use crate::production::Productions;
    pub use crate::state::State as a;
    pub use crate::state::StateId as i;
    pub use crate::state::StateId;
    pub use crate::state::States;
    pub use crate::symbol::Symbol;
    pub use crate::symbol::Symbol::NONTERMINAL as N;
    pub use crate::symbol::Symbol::TERMINAL as T;
    pub use crate::symbol::SymbolId as s;
    pub use crate::symbol::SymbolId;
    pub use crate::symbol::Symbols;
    pub use indexmap::IndexMap;
    pub use indexmap::IndexSet;
    pub use quote::quote;
    pub use std::rc::Rc;
}

#[cfg(test)]
mod tests;
