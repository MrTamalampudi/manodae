use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::ToTokens,
    production::{Production, ProductionId, Productions},
};

impl<AST, Tokens, TranslatorStack> ToTokens for Production<AST, Tokens, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let error_message = match &self.error_message {
            Some(err) => quote! {Some(String::new(#err))},
            None => quote! {None},
        };
        let action = if self.action_tokens.is_empty() {
            quote! {None}
        } else {
            self.action_tokens.clone()
        };
        let action_tokens = quote! {None};
        let body: Vec<_> = self
            .body
            .iter()
            .map(|sym| {
                let tokens = sym.to_tokens();
                quote! {#tokens}
            })
            .collect();
        let index = &self.index;
        let head = &self.head.to_tokens();
        let production = quote! {
            P::n(
                #index,
                #head,
                Vec::new([#(#body),*]),
                #error_message,
                #action_tokens,
                #action,
            )
        };
        production
    }
}

impl ToTokens for ProductionId {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let id = self.0;
        quote! {PID(#id)}
    }
}

impl<AST, Tokens, TranslatorStack> ToTokens for Productions<AST, Tokens, TranslatorStack> {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let map: Vec<TokenStream> = self
            .map
            .iter()
            .map(|(_key, value)| {
                let index = value.0;
                let key = quote! { pc!(#index)};
                let value = value.to_tokens();
                quote! {(#key,#value)}
            })
            .collect();
        //let vec_: Vec<TokenStream> = self.vec.iter().map(|symbol| symbol.to_tokens()).collect();
        let productions = quote! {
            Productions {
                map: IndexMap::from([#(#map),*]),
                vec: productions,
            }
        };
        productions
    }
}
