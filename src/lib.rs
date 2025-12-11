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

pub use codegen::codegen;

pub mod prelude {
    pub use crate::codegen;
    pub use crate::error::ParseError;
    pub use crate::grammar;
    pub use crate::grammar::Grammar;
    pub use crate::interner::Interner;
    pub use crate::parser::LR1_Parser;
    pub use crate::production::Production;
    pub use crate::symbol::Symbol;
    pub use crate::symbol::SymbolId;
    pub use std::rc::Rc;
}

#[cfg(test)]
mod tests;
