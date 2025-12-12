use std::{
    fmt::Debug,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    process::Command,
    str::FromStr,
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

pub struct Codegen<AST, Token, TranslatorStack> {
    path: PathBuf,
    grammar: Grammar<AST, Token, TranslatorStack>,
}

const FOLDER: &str = "parser_generated";
const F_ACTION: &str = "action.rs";
const F_GRAMMAR: &str = "grammar.rs";
const F_LR: &str = "lr.rs";
const F_GOTO: &str = "goto.rs";
const F_PARSER: &str = "parser.rs";
const F_FIRST: &str = "first.rs";
const F_FOLLOW: &str = "follow.rs";
const F_HASH: &str = "hash.txt";

impl<AST, Token, TranslatorStack> Codegen<AST, Token, TranslatorStack>
where
    AST: Debug + PartialEq + Clone,
    Token: Debug + PartialEq + Clone + ToString,
    TranslatorStack: Debug + PartialEq + Clone,
{
    pub fn gen(path: PathBuf, grammar: Grammar<AST, Token, TranslatorStack>) {
        let path = path.parent().unwrap().to_path_buf();
        let mut codegen = Codegen {
            path: path,
            grammar: grammar,
        };
        codegen.mkdir();
        let hash = codegen.grammar_hash();
        if !codegen.needs_regen(hash) {
            return;
        }
        let lr = LR1_Parser::new(codegen.grammar.clone());
        codegen.write_grammar(lr.grammar.clone());
        codegen.write_LR1_Automata(lr.LR1_automata.clone());
        codegen.write_first_follow_set(lr.first_set.clone(), "first");
        codegen.write_first_follow_set(lr.follow_set.clone(), "follow");
        codegen.write_action(lr.action.clone());
        codegen.write_goto(lr.goto.clone());
        codegen.write_parser();
        codegen.write_hash(hash);

        codegen.rustfmt();
    }

    fn grammar_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.grammar.hash(&mut s);
        s.finish()
    }

    fn needs_regen(&self, hash: u64) -> bool {
        if !self.path.join(F_HASH).exists() {
            return true;
        }
        let current_hash = hash;
        let hash_file_path = self.path.join(F_HASH);
        let file = fs::read_to_string(hash_file_path);
        let file_unwrapped = file.unwrap();
        let previous_hash = file_unwrapped.lines().next().unwrap();
        let previous_hash = <u64>::from_str(previous_hash).unwrap();
        current_hash != previous_hash
    }

    fn mkdir(&mut self) {
        if self.path.join(FOLDER).exists() {
            self.path = self.path.join(FOLDER);
            return;
        }
        let mut command = Command::new("mkdir");
        command.arg(FOLDER);
        command.current_dir(&self.path);
        command.spawn().expect("Failed to create codegen folder");
        self.path = self.path.join(FOLDER);
    }

    fn rustfmt(&self) {
        Command::new("rustfmt")
            .args([
                F_GRAMMAR, F_FIRST, F_FOLLOW, F_LR, F_ACTION, F_GOTO, F_PARSER,
            ])
            .current_dir(&self.path)
            .spawn()
            .expect("Failed to format generated rust code");
    }

    fn write_hash(&self, hash: u64) {
        let mut file = File::create(self.path.join(F_HASH)).unwrap();
        file.write_all(hash.to_string().as_bytes()).ok();
    }

    fn write_parser(&self) {
        let code = quote! {
            include!(#F_GRAMMAR);
            include!(#F_LR);
            include!(#F_FIRST);
            include!(#F_FOLLOW);
            include!(#F_ACTION);
            include!(#F_GOTO);

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
        let mut file = File::create(self.path.join(F_PARSER)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_grammar(&self, grammar: Grammar<AST, Token, TranslatorStack>) {
        let grammar = grammar.to_tokens();
        let closure_macro = closure_macro();
        let symbols_clone_macro = symbol_clone_macro();
        let productions_clone_macro = production_clone_macro();
        let string_from_macro = string_from_macro();
        let code = quote! {
            #symbols_clone_macro
            #productions_clone_macro
            #closure_macro
            #string_from_macro

            fn grammar() -> Grammar<AST, Token, TranslatorStack> {
                #grammar
            }
        };
        let code_s = code.to_string();
        let mut file = File::create(self.path.join(F_GRAMMAR)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_LR1_Automata(&self, lr: States) {
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
        let mut file = File::create(self.path.join(F_LR)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_first_follow_set(&self, set: IndexMap<SymbolId, IndexSet<SymbolId>>, name: &str) {
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
        let mut file = File::create(self.path.join(filename)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_action(&self, action: IndexMap<StateId, IndexMap<SymbolId, Action>>) {
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
        let mut file = File::create(self.path.join(F_ACTION)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_goto(&self, goto: IndexMap<StateId, IndexMap<SymbolId, StateId>>) {
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
        let mut file = File::create(self.path.join(F_GOTO)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }
}

// fn write_parser(parser: LR1_Parser<AST, Token, TranslatorStack>) {}
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
