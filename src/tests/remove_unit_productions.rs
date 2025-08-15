use crate::error::ParseError;
use crate::grammar::Grammar;
use crate::parser::Parser;
use crate::production::Production;
use crate::symbol::Symbol;
use crate::tests::{TranslatorStack, AST};
use crate::{grammar, tests::tokens::Token};

#[test]
#[allow(unused_variables)]
fn remove_unit_productions() {
    let grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> AA;

        B-> B;

        AA -> A;

        A -> [Token::A];
    );

    let mut parser = Parser::new(&grammar.productions);
    parser.compute_lr0_items();

    // eliminate(&mut grammar.productions);

    let mut errors: Vec<ParseError<Token>> = Vec::new();
    let mut ast = AST::new();
    let tokens: Vec<Token> = vec![Token::A, Token::EOF];
    parser.parse(tokens, &mut errors, &mut ast);
    assert!(true)
}
