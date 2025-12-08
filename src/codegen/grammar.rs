use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, grammar::Grammar};

impl<AST, Token, TranslatorStack> ToTokens for Grammar<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let nt: Vec<_> = self
            .non_terminals
            .iter()
            .map(|sym| {
                let tokens = sym.to_tokens();
                quote! {Rc::new(#tokens)}
            })
            .collect();
        let nt = quote! {IndexSet::from([#(Rc::new(#nt)),*])};
        let t: Vec<_> = self
            .terminals
            .iter()
            .map(|sym| {
                let tokens = sym.to_tokens();
                quote! {Rc::new(#tokens)}
            })
            .collect();
        let t = quote! {IndexSet::from([#(Rc::new(#t)),*])};
        let start = &self.start.to_tokens();
        let start = quote! {Rc::new(#start)};
        let productions: Vec<_> = self
            .productions
            .iter()
            .map(|prod| {
                let tokens = prod.to_tokens();
                quote! {Rc::new(#tokens)}
            })
            .collect();
        let productions = quote! {IndexSet::from([#(#productions),*])};
        let production_head_map = quote! {IndexMap::new()};

        let grammar = quote! {
            G {
                non_terminals:#nt,
                terminals:#t,
                start:#start,
                productions:#productions,
                production_head_map:#production_head_map
            }
        };
        grammar
    }
}
