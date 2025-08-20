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
fn basic() {
    let grammar: Grammar<AST, Token, TranslatorStack> = grammar!(
        Start -> Expression;

        Expression  -> LiteralExpression
        | OperatorExpression;

        LiteralExpression -> [Token::A] | [Token::B];

        OperatorExpression -> NegationExpression;

        NegationExpression -> [Token::C] Expression;

        // ComparisionExpression -> Expression Equality Expression;
    );
    let mut parser = LR1_Parser::new(&grammar);
    parser.items();
    // eliminate(&mut grammar.productions);

    // let mut errors: Vec<ParseError<Token>> = Vec::new();
    // let mut ast = AST::new();
    // let tokens: Vec<Token> = vec![Token::A, Token::EOF];
    // parser.parse(tokens, &mut errors, &mut ast);
    assert!(true)
}
