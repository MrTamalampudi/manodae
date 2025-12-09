use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, grammar::Grammar};

// todo add symbols field to G
impl<AST, Token, TranslatorStack> ToTokens for Grammar<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
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
        let symbols = self.symbols.to_tokens();

        let grammar = quote! {
            G {
                start:#start,
                productions:#productions,
                production_head_map:#production_head_map
                symbols:#symbols
            }
        };
        grammar
    }
}
