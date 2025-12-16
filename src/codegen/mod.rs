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
use proc_macro2::{Ident, TokenStream};
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
    generics: Vec<Ident>,
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
    pub fn gen(
        path: PathBuf,
        grammar: Grammar<AST, Token, TranslatorStack>,
        generics: [&'static str; 3],
    ) {
        let mut codegen = Codegen {
            path: path,
            grammar: grammar,
            generics: generics
                .iter()
                .map(|g| format_ident!("{}", g))
                .collect::<Vec<Ident>>(),
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
        let [a, t, ts] = [&self.generics[0], &self.generics[1], &self.generics[2]];
        let code = quote! {
            include!(#F_GRAMMAR);
            include!(#F_LR);
            include!(#F_FIRST);
            include!(#F_FOLLOW);
            include!(#F_ACTION);
            include!(#F_GOTO);



            fn get_parser() -> LR1_Parser<#a,#t ,#ts> {
                LR1_Parser {
                    grammar: __grammar__(),
                    LR1_automata: __lr__(),
                    follow_set: __follow__(),
                    first_set: __first__(),
                    conflicts: false,
                    goto: __goto__(),
                    action: __action__(),
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
        let [a, t, ts] = [&self.generics[0], &self.generics[1], &self.generics[2]];
        let grammar = grammar.to_tokens();
        let f = string_from_macro();
        let s = symbol_clone_macro();
        let d = production_clone_macro();
        let code = quote! {
            #f
            #s
            #d

            #[allow(unused_variables)]
            fn __grammar__() -> Grammar<#a,#t ,#ts> {
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

            pub fn __lr__() -> States {
                #states
            }
        };
        let code_s = code.to_string();
        let mut file = File::create(self.path.join(F_LR)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_first_follow_set(&self, set: IndexMap<SymbolId, IndexSet<SymbolId>>, name: &str) {
        let isf = indexset_from_macro();
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
                let value = quote! {h!{[#(#value),*]}};
                let key = key.to_tokens();
                quote! {(#key,#value)}
            })
            .collect();
        let file_name = format_ident!("__{}__", name);
        let code = quote! {
            #isf
            pub fn #file_name() -> IndexMap<SymbolId, IndexSet<SymbolId>> {
                IndexMap::from([#(#set),*])
            }
        };

        let filename = format!("{name}.rs");
        let code_s = code.to_string();
        let mut file = File::create(self.path.join(filename)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_action(&self, action: IndexMap<StateId, IndexMap<SymbolId, Action>>) {
        let imf = indexmap_from_macro();
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
                let value = quote! {{g!{[#(#value),*]}}};
                quote! {(#key,#value)}
            })
            .collect();
        let code = quote! {
            #imf
            pub fn __action__() -> IndexMap<StateId, IndexMap<SymbolId, Action>> {
                IndexMap::from([#(#action),*])
            }
        };
        let code_s = code.to_string();
        let mut file = File::create(self.path.join(F_ACTION)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }

    fn write_goto(&self, goto: IndexMap<StateId, IndexMap<SymbolId, StateId>>) {
        let imf = indexmap_from_macro();
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
                //g! expands to IndexMap::from(..);
                let value = quote! {{g!{[#(#value),*]}}};
                quote! {(#key,#value)}
            })
            .collect();
        let code = quote! {
            #imf
            pub fn __goto__() -> IndexMap<StateId, IndexMap<SymbolId, StateId>> {
                IndexMap::from([#(#goto),*])
            }
        };
        let code_s = code.to_string();
        let mut file = File::create(self.path.join(F_GOTO)).unwrap();
        file.write_all(code_s.as_bytes()).ok();
    }
}

// fn write_parser(parser: LR1_Parser<AST, Token, TranslatorStack>) {}
fn production_clone_macro() -> TokenStream {
    quote! {
        macro_rules! d {
            ($idx:expr,$p:ident) => {
                $p[$idx].clone()
            };
        }
    }
}
fn symbol_clone_macro() -> TokenStream {
    quote! {
        macro_rules! e {
            ($idx:expr,$s:ident) => {
                $s[$idx].clone()
            };
        }
    }
}
fn string_from_macro() -> TokenStream {
    quote! {
        macro_rules! f {
            ($s:expr) => {
                String::from($s)
            };
        }
    }
}
fn indexmap_from_macro() -> TokenStream {
    quote! {
        macro_rules! g {
            ($s:expr) => {
                IndexMap::from($s)
            };
        }
    }
}
fn indexset_from_macro() -> TokenStream {
    quote! {
        macro_rules! h {
            ($s:expr) => {
                IndexSet::from($s)
            };
        }
    }
}
