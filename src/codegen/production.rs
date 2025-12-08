use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, production::Production};

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
        let action_tokens = TokenStream::new();
        let body: Vec<_> = self
            .body
            .iter()
            .map(|sym| {
                let tokens = sym.to_tokens();
                quote! {Rc::new(#tokens)}
            })
            .collect();
        let index = &self.index;
        let head = &self.head;
        let production = quote! {
            Production {
                index: #index,
                head: #head,
                error_message: #error_message,
                action_tokens: #action_tokens,
                action: #action,
                body: Vec::new([#(#body),*]),
            }
        };
        production
    }
}
