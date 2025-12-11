use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::ToTokens,
    symbol::{Symbol, SymbolId, Symbols},
};

impl ToTokens for Symbol {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let t = match self {
            //T(sf!(#terminal)) expands to TERMINAL(String::from("Some"))
            Symbol::TERMINAL(terminal) => quote! { T(sf!(#terminal))},
            //NT(sf!(#terminal)) expands to NONTERMINAL(String::from("Some"))
            Symbol::NONTERMINAL(terminal) => quote! { NT(sf!(#terminal))},
        };
        t
    }
}

impl ToTokens for SymbolId {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let id = self.0;
        //expands to SymbolId(id)
        quote! {s(#id)}
    }
}

impl ToTokens for Symbols {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let map: Vec<TokenStream> = self
            .map
            .iter()
            .map(|(_key, value)| {
                let index = value.0;
                let key = quote! { sc!(#index)};
                let value = value.to_tokens();
                quote! {(#key,#value)}
            })
            .collect();
        // let vec_: Vec<TokenStream> = self.vec.iter().map(|symbol| symbol.to_tokens()).collect();
        let terminals: Vec<TokenStream> =
            self.terminals.iter().map(|sid| sid.to_tokens()).collect();
        let non_terminals: Vec<TokenStream> = self
            .non_terminals
            .iter()
            .map(|sid| sid.to_tokens())
            .collect();
        let symbols = quote! {
            Symbols {
                map: IndexMap::from([#(#map),*]),
                vec: symbols,
                terminals: vec![#(#terminals),*],
                non_terminals: vec![#(#non_terminals),*],
            }
        };
        symbols
    }
}
