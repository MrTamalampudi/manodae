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
fn left_factor() {
    let grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> E;

        E  -> LE | OE;

        LE -> [Token::A];

        OE -> NE | CE;

        NE -> [Token::C] E;

        CE -> E D E| E F E;

        D -> [Token::D];

        F -> [Token::F];
    );
    let mut parser = LR1_Parser::new(&grammar);
    parser.construct_LALR_Table();
    // eliminate(&mut grammar.productions);

    // let mut errors: Vec<ParseError<Token>> = Vec::new();
    // let mut ast = AST::new();
    // let tokens: Vec<Token> = vec![Token::A, Token::EOF];
    // parser.parse(tokens, &mut errors, &mut ast);
    assert!(true)
}
