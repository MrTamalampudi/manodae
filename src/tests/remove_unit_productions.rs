use crate::error::ParseError;
use crate::grammar::Grammar;
use crate::parser::Parser;
use crate::production::Production;
use crate::symbol::Symbol;
use crate::tests::{TranslatorStack, AST};
use crate::{grammar, tests::tokens::Token};
use std::sync::Arc;

#[test]
#[allow(unused_variables)]
fn remove_unit_productions() {
    let mut grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> AA;

        AA -> A;

        A -> [Token::A];
        B -> [Token::B];
        C -> [Token::C];
        D -> [Token::D];
        E -> [Token::E];
        F -> [Token::F];
        G -> [Token::G];
        H -> [Token::H];
    );

    let mut parser = Parser::new(&mut grammar.productions);
    parser.compute_lr0_items();

    // eliminate(&mut grammar.productions);

    let mut errors: Vec<ParseError<Token>> = Vec::new();
    let mut ast = AST::new();
    // let tokens: Vec<Token> = vec![Token::A, Token::B, Token::C, Token::EOF];
    // parser.parse(tokens, &mut errors, &mut ast);
    assert!(true)
}
