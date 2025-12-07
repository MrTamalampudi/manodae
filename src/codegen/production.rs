use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::production::Production;

impl<AST, Tokens, TranslatorStack> ToTokens for Production<AST, Tokens, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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
        let body = self.body.clone();
        let production = quote! {
            Production {
                index: #self.index,
                head: #self.head,
                error_message: #error_message,
                action_tokens: #action_tokens,
                action: #action,
                body: Vec::new([#(Rc::new(#body)),*]),
            }
        };
        tokens.extend(production);
    }
}
