use std::fmt::Debug;

use grammar::Grammar;
use parser::Parser;
use production::Production;
use symbol::Symbol;
use terminal::Terminal;

pub mod action;
pub mod first;
pub mod follow;
pub mod grammar;
mod parser;
pub mod production;
pub mod state;
pub mod symbol;
pub mod terminal;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    A,
    B,
    C,
    EOF,
}

impl Terminal for TokenType {
    fn get_ending_token() -> TokenType {
        TokenType::EOF
    }
}

fn main() {
    let grammar: Grammar<TokenType> = crate::grammar!(
        TokenType,
        S -> TokenType::A [E];
        E -> TokenType::B [Z] [Z];
        Z -> TokenType::C
    );
    //println!("{:#?}", grammar.productions);
    let mut p = Parser::new(grammar.productions);
    p.compute_lr0_items();
    let input = vec![
        TokenType::A,
        TokenType::B,
        TokenType::C,
        TokenType::C,
        TokenType::EOF,
    ];
    p.parse(input);
}
