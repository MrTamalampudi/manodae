use std::{fs::File, io::Write};

use proc_macro2::TokenStream;
use quote::quote;

use crate::parser::LR1_Parser;

mod action;
mod grammar;
mod item;
mod parser;
mod production;
mod state;
mod symbol;

pub fn codegen<AST, Token, TranslatorStack>(parser: LR1_Parser<AST, Token, TranslatorStack>) {
    let parser = parser.to_tokens();
    let code = quote! {
        use manodae::parser::LR1_Parser;
        use manodae::symbol::Symbol;
        fn get_parser() -> LR1_Parser<AST, Token, TranslatorStack> {
            let parser = #parser
        }
    };
    let code_s = code.to_string();
    let mut file = File::create("parser.rs").unwrap();
    file.write_all(code_s.as_bytes()).ok();
}

pub trait ToTokens {
    fn to_tokens(&self) -> TokenStream;
}
