use std::{
    fmt::Debug,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    action::Action,
    grammar::Grammar,
    parser::LR1_Parser,
    state::{StateId, States},
    symbol::SymbolId,
};

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

pub fn codegen<AST, Token, TranslatorStack>(
    path: PathBuf,
    parser: LR1_Parser<AST, Token, TranslatorStack>,
) where
    AST: Clone + Debug + PartialEq,
    Token: Clone + Debug + PartialEq + ToString,
    TranslatorStack: Clone + Debug + PartialEq,
{
    let mut command = Command::new("mkdir");
    command.arg("parser_generated");
    command.current_dir(path.parent().unwrap());
    command.spawn().expect("msg");
    let pg = path.parent().unwrap().join("parser_generated");
    write_grammar(parser.grammar.clone(), pg.clone());
    write_LR1_Automata(parser.LR1_automata.clone(), pg.clone());
    write_first_follow_set(parser.first_set.clone(), "first", pg.clone());
    write_first_follow_set(parser.follow_set.clone(), "follow", pg.clone());
    write_action(parser.action.clone(), pg.clone());
    write_goto(parser.goto.clone(), pg);
    //let parser = parser.to_tokens();
    println!("pathhhhhhhhhhhh  {:#?}", path);

    let code = quote! {
        use manodae::parser::LR1_Parser;
        use indexmap::IndexMap;

        include!("grammar.rs");
        include!("lr.rs");
        include!("first.rs");
        include!("follow.rs");
        include!("action.rs");
        include!("goto.rs");

        fn get_parser() -> LR1_Parser<AST, Token, TranslatorStack> {
            LR1_Parser {
                grammar: grammar(),
                LR1_automata: lr(),
                follow_set: follow(),
                first_set: first(),
                conflicts: false,
                goto: goto(),
                action: action(),
                //used only when constructing table, no need for parsing
                item_closure_map: IndexMap::new(),
                //used only when constructing table, no need for parsing
                closure_map: IndexMap::new(),
            }
        }
    };
    let code_s = code.to_string();
    let mut file = File::create("parser.rs").unwrap();
    file.write_all(code_s.as_bytes()).ok();
    let _ = Command::new("rustfmt")
        .args([
            "grammar.rs",
            "first.rs",
            "follow.rs",
            "lr.rs",
            "action.rs",
            "goto.rs",
            "parser.rs",
        ])
        .spawn();
}

fn write_grammar<AST, Token, TranslatorStack>(
    grammar: Grammar<AST, Token, TranslatorStack>,
    path: PathBuf,
) {
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
    let mut file = File::create(path.join("grammar.rs")).unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

pub fn write_LR1_Automata(lr: States, path: PathBuf) {
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
    let mut file = File::create(path.join("lr.rs")).unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

fn write_first_follow_set(set: IndexMap<SymbolId, IndexSet<SymbolId>>, name: &str, path: PathBuf) {
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
    let mut file = File::create(path.join(filename)).unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

fn write_action(action: IndexMap<StateId, IndexMap<SymbolId, Action>>, path: PathBuf) {
    let indexmap_from = indexmap_from_macro();
    let action: Vec<_> = action
        .iter()
        .map(|(key, value)| {
            let value: Vec<_> = value
                .iter()
                .map(|(key, value)| {
                    let key = key.to_tokens();
                    let value = value.to_tokens();
                    quote! {(#key,#value)}
                })
                .collect();
            let key = key.to_tokens();
            //m! expands to IndexMap::from(..);
            let value = quote! {m!([#(#value),*])};
            quote! {(#key,#value)}
        })
        .collect();
    let code = quote! {
        use indexmap::{IndexMap, IndexSet};
        use manodae::state::StateId as i;
        use manodae::action::Action::SHIFT as S;
        use manodae::action::Action::REDUCE as R;
        use manodae::action::Action::ERROR as E;
        use manodae::action::Action::ACCEPT as A;
        use manodae::symbol::SymbolId as s;

        #indexmap_from

        fn action() -> IndexMap<StateId, IndexMap<SymbolId, Action>> {
            IndexMap::from([#(#action),*])
        }
    };
    let code_s = code.to_string();
    let mut file = File::create(path.join("action.rs")).unwrap();
    file.write_all(code_s.as_bytes()).ok();
}
fn write_goto(goto: IndexMap<StateId, IndexMap<SymbolId, StateId>>, path: PathBuf) {
    let indexmap_from = indexmap_from_macro();
    let goto: Vec<_> = goto
        .iter()
        .map(|(key, value)| {
            let value: Vec<_> = value
                .iter()
                .map(|(key, value)| {
                    let key = key.to_tokens();
                    let value = value.to_tokens();
                    quote! {(#key,#value)}
                })
                .collect();
            let key = key.to_tokens();
            //m! expands to IndexMap::from(..);
            let value = quote! {m!([#(#value),*])};
            quote! {(#key,#value)}
        })
        .collect();
    let code = quote! {
        use indexmap::{IndexMap, IndexSet};
        use manodae::state::StateId as i;
        use manodae::symbol::SymbolId as s;

        #indexmap_from

        fn action() -> IndexMap<StateId, IndexMap<SymbolId, Action>> {
            IndexMap::from([#(#goto),*])
        }
    };
    let code_s = code.to_string();
    let mut file = File::create(path.join("goto.rs")).unwrap();
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
fn indexmap_from_macro() -> TokenStream {
    quote! {
        macro_rules! m {
            ($s:expr) => {
                IndexMap::from($s)
            };
        }
    }
}
