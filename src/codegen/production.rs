use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::ToTokens,
    production::{Production, ProductionId, Productions},
};

impl<AST, Tokens, TranslatorStack> ToTokens for Production<AST, Tokens, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let error_message = match &self.error_message {
            //sf!(err) expands to String::from(err)
            Some(err) => quote! {Some(f!{#err})},
            None => quote! {None},
        };
        let action = if self.action_tokens.is_empty() {
            quote! {None}
        } else {
            self.action_tokens.clone()
        };
        //q!{} expands to quote!{}
        let action_tokens = quote! {quote!{}};
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
                vec![#(#body),*],
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
        //expands to ProductionId(id)
        quote! {p(#id)}
    }
}

impl<AST, Tokens, TranslatorStack> ToTokens for Productions<AST, Tokens, TranslatorStack> {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let map: Vec<TokenStream> = self
            .map
            .iter()
            .map(|(_key, value)| {
                let index = value.0;
                //pc!(index) => production[index].clone()
                let key = quote! { d!{#index,y}};
                let value = value.to_tokens();
                quote! {(#key,#value)}
            })
            .collect();
        let productions = quote! {
            Productions {
                map: IndexMap::from([#(#map),*]),
                vec: y,
            }
        };
        productions
    }
}
