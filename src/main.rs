use std::collections::HashMap;

use first::compute_first_set;
use follow::compute_follow_set;
use parser::Parser;
use production::Production;
use symbol::{unique_symbols, Symbol};

pub mod first;
pub mod follow;
mod grammar;
mod parser;
pub mod production;
pub mod symbol;

#[derive(Debug, Clone)]
struct State {
    state: usize,
    productions: Vec<Production>,
    transition_symbol: Symbol,
    action: HashMap<TokenType, Action>,
    goto: HashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    SHIFT(usize),
    REDUCE(usize),
    ACCEPT,
    ERROR(String),
}

#[derive(Debug)]
struct Grammar {
    productions: Vec<Production>,
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            productions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    A,
    B,
    C,
    EOF,
}

fn main() {
    let grammar: Grammar = crate::grammar!(
        S -> TokenType::A [E];
        E -> TokenType::B [Z] [Z];
        Z -> TokenType::C
    );
    //println!("{:#?}", grammar.productions);
    Parser::new(grammar.productions).compute_lr0_items();
}
