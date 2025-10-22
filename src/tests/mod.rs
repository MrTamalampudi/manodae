#![allow(dead_code)]
// mod E1;
// mod basic;
// mod left_factor;
// mod tokens;

#[derive(Debug, Clone, PartialEq)]
struct AST {
    pub result: bool,
}

impl AST {
    pub fn new() -> AST {
        AST { result: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslatorStack {}
