use crate::error::ParseError;
use crate::grammar::Grammar;
use crate::parser::Parser;
use crate::production::Production;
use crate::symbol::Symbol;
use crate::tests::{TranslatorStack, AST};
use crate::{grammar, tests::tokens::Token};
use std::sync::Arc;

// #[test]
#[allow(unused_variables)]
fn basic() {
    let grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> A B C
        {action:|ast,token_stack,tl_stack,errors| {
            set_result(ast,true);
        }}
        ;

        A -> [Token::A];
        B -> [Token::B];
        C -> [Token::C];
        D -> [Token::D];
        E -> [Token::E];
        F -> [Token::F];
        G -> [Token::G];
        H -> [Token::H];
    );

    let mut parser = Parser::new(&grammar.productions);
    parser.compute_lr0_items();

    let mut errors: Vec<ParseError<Token>> = Vec::new();
    let mut ast = AST::new();

    let tokens: Vec<Token> = vec![Token::A, Token::B, Token::C, Token::EOF];
    parser.parse(tokens, &mut errors, &mut ast);
    assert!(ast.result)
}

fn set_result(ast: &mut AST, result: bool) {
    ast.result = result
}
