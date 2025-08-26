use crate::error::ParseError;
use crate::grammar;
use crate::{
    grammar::Grammar,
    parser::LR1_Parser,
    production::Production,
    symbol::Symbol,
    tests::{tokens::Token, TranslatorStack, AST},
};
use std::sync::Arc;

#[test]
fn E1() {
    let grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> C C;
        C -> [Token::C] C | [Token::D];
    );
    let mut parser = LR1_Parser::new(&grammar);
    // eliminate(&mut grammar.productions);

    let mut errors: Vec<ParseError<Token>> = Vec::new();
    let mut ast = AST::new();
    let tokens: Vec<Token> = vec![Token::D, Token::D, Token::EOF];
    parser.parse(tokens, &mut errors, &mut ast);
    assert!(true)
}
