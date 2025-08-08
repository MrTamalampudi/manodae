mod basic;
mod tokens;

#[derive(Debug, Clone)]
struct AST {
    pub result: bool,
}

impl AST {
    pub fn new() -> AST {
        AST { result: false }
    }
}

#[derive(Debug, Clone)]
pub struct TranslatorStack {}
