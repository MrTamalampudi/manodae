use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, grammar::Grammar};

// todo add symbols field to G
impl<AST, Token, TranslatorStack> ToTokens for Grammar<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let start = &self.start.to_tokens();
        let start = quote! {#start};
        let production_head_map = quote! {IndexMap::new()};

        let symbols_vec: Vec<TokenStream> = self
            .symbols
            .vec
            .iter()
            .map(|symbol| symbol.to_tokens())
            .collect();
        let symbols = self.symbols.to_tokens();

        let productions_vec: Vec<TokenStream> = self
            .productions
            .vec
            .iter()
            .map(|productions| productions.to_tokens())
            .collect();
        let productions = self.productions.to_tokens();

        let grammar = quote! {
            let y = vec![#(#productions_vec),*];
            let x = vec![#(#symbols_vec),*];
            Grammar {
                start:#start,
                productions:#productions,
                production_head_map:#production_head_map,
                symbols:#symbols
            }
        };
        grammar
    }
}
