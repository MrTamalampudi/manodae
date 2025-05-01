use std::fmt::Debug;

use grammar::Grammar;
use parser::Parser;
use production::Production;
use symbol::Symbol;
use terminal::Terminal;

pub mod action;
pub mod conflict;
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
    C(String),
    EOF,
}

impl Terminal for TokenType {
    fn get_ending_token() -> String {
        TokenType::EOF.to_string()
    }
    fn to_string(&self) -> String {
        match self {
            TokenType::A => String::from("A"),
            TokenType::B => String::from("B"),
            TokenType::C(_) => String::from("C"),
            TokenType::EOF => String::from("EOF"),
        }
    }
}

impl Terminal for String {
    fn get_ending_token() -> String {
        TokenType::get_ending_token()
    }
}

fn main() {
    let dummy = String::new();
    let grammar: Grammar = crate::grammar!(
        TokenType,
        S -> A B
        |B C;
        A -> [TokenType::A];
        B -> [TokenType::A];
        C -> [TokenType::B]
    );
    let mut p = Parser::new(grammar.productions);
    p.compute_lr0_items();
    //println!("{:#?}", p.conflicts);
    let input = vec![
        TokenType::A,
        TokenType::C("dumm".to_string()),
        TokenType::EOF,
    ];
    p.parse(input);
}
