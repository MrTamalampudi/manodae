use std::{fs::File, io::Write};

use proc_macro2::TokenStream;
use quote::quote;

use crate::{grammar::Grammar, parser::LR1_Parser, state::States};

mod action;
mod grammar;
mod item;
mod parser;
mod production;
mod state;
mod symbol;

pub trait ToTokens {
    fn to_tokens(&self) -> TokenStream;
}

pub fn codegen<AST, Token, TranslatorStack>(parser: LR1_Parser<AST, Token, TranslatorStack>)
where
    AST: Clone,
    Token: Clone,
    TranslatorStack: Clone,
{
    write_grammar(parser.grammar.clone());
    write_LR1_Automata(parser.LR1_automata.clone());
    //let parser = parser.to_tokens();
    let code = quote! {
        use manodae::parser::LR1_Parser as L;
        use manodae::symbol::Symbol as C;
        use manodae::symbol::SymbolId as SID;
        use manodae::state::State as S;
        use manodae::item::Item as I;
        use manodae::grammar::Grammar as G;
        use manodae::action::Action as A;
        use manodae::production::Production as P;

        fn get_parser() -> LR1_Parser<AST, Token, TranslatorStack> {
            //let parser = #parser
        }
    };
    let code_s = code.to_string();
    let mut file = File::create("parser.rs").unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

fn write_grammar<AST, Token, TranslatorStack>(grammar: Grammar<AST, Token, TranslatorStack>) {
    let grammar = grammar.to_tokens();
    let code = quote! {
        use manodae::production::Production as P;
        use manodae::symbol::Symbol as C;
        use manodae::symbol::SymbolId as SID;
        use manodae::grammar::Grammar;
        use indexmap::IndexMap;
        use std::rc::Rc;

        macro_rules! sc {
            ($idx:expr) => {
                symbols[$idx].clone()
            };
        }

        macro_rules! pc {
            ($idx:expr) => {
                productions[$idx].clone()
            };
        }

        fn grammar() -> Grammar<AST, Token, TranslatorStack> {
            #grammar
        }
    };
    let code_s = code.to_string();
    let mut file = File::create("grammar.rs").unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

pub fn write_LR1_Automata(lr: States) {
    let states = lr.to_tokens();
    let code = quote! {
        use manodae::production::Production as P;
        use manodae::symbol::Symbol as C;
        use manodae::symbol::SymbolId as SID;
        use manodae::grammar::Grammar;
        use indexmap::IndexMap;
        use std::rc::Rc;

        macro_rules! sc {
            ($idx:expr) => {
                states[$idx].clone()
            };
        }

        fn lr() -> States {
            #states
        }
    };
    let code_s = code.to_string();
    let mut file = File::create("lr.rs").unwrap();
    file.write_all(code_s.as_bytes()).ok();
}
