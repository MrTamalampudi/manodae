use std::{fs::File, io::Write, process::Command};

use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{grammar::Grammar, parser::LR1_Parser, state::States, symbol::SymbolId};

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
    write_first_follow_set(parser.first_set.clone(), "first");
    write_first_follow_set(parser.follow_set.clone(), "follow");
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
    let _ = Command::new("rustfmt")
        .args(["grammar.rs", "first.rs", "follow.rs", "lr.rs"])
        .spawn();
}

fn write_grammar<AST, Token, TranslatorStack>(grammar: Grammar<AST, Token, TranslatorStack>) {
    let grammar = grammar.to_tokens();
    let closure_macro = closure_macro();
    let symbols_clone_macro = symbol_clone_macro();
    let productions_clone_macro = production_clone_macro();
    let string_from_macro = string_from_macro();
    let code = quote! {
        use manodae::production::Production as P;
        use manodae::production::ProductionId as p;
        use manodae::symbol::Symbol::TERMINAL as T;
        use manodae::symbol::Symbol::NONTERMINAL as NT;
        use manodae::symbol::SymbolId as s;
        use manodae::grammar::Grammar;
        use indexmap::IndexMap;
        use std::rc::Rc;

        #symbols_clone_macro
        #productions_clone_macro
        #closure_macro
        #string_from_macro

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
        use indexmap::IndexMap;
        use manodae::grammar::Grammar;
        use manodae::item::Item as I;
        use manodae::production::ProductionId as p;
        use manodae::state::State as S;
        use manodae::state::StateId as i;
        use manodae::state::States;
        use manodae::symbol::SymbolId as s;

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

fn write_first_follow_set(set: IndexMap<SymbolId, IndexSet<SymbolId>>, name: &str) {
    let set: Vec<_> = set
        .iter()
        .map(|(key, value)| {
            let value: Vec<_> = value
                .iter()
                .map(|sym| {
                    let tokens = sym.to_tokens();
                    quote! {#tokens}
                })
                .collect();
            let value = quote! {IndexSet::from([#(#value),*])};
            let key = key.to_tokens();
            quote! {(#key,#value)}
        })
        .collect();
    let file_name = format_ident!("{}", name);
    let code = quote! {
        use manodae::symbol::SymbolId as s;
        use indexmap::IndexMap;
        use indexmap::IndexSet;
        fn #file_name() -> IndexMap<SymbolId, IndexSet<SymbolId>> {
            IndexMap::from([#(#set),*])
        }
    };

    let filename = format!("{name}.rs");
    let code_s = code.to_string();
    let mut file = File::create(filename).unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

fn closure_macro() -> TokenStream {
    quote! {
        macro_rules! c {
            ($b:block) => {
                Rc::new(|ast, token_stack, tl_stack, errors| $b)
            };
        }
    }
}
fn production_clone_macro() -> TokenStream {
    quote! {
        macro_rules! pc {
            ($idx:expr) => {
                productions[$idx].clone()
            };
        }
    }
}
fn symbol_clone_macro() -> TokenStream {
    quote! {
        macro_rules! sc {
            ($idx:expr) => {
                symbols[$idx].clone()
            };
        }
    }
}
fn string_from_macro() -> TokenStream {
    quote! {
        macro_rules! sf {
            ($s:expr) => {
                String::from($s)
            };
        }
    }
}
