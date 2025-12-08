use proc_macro2::TokenStream;
use quote::quote;

use crate::{action::Action, codegen::ToTokens};

impl<AST, Token, TranslatorStack> ToTokens for Action<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        match self {
            Action::SHIFT(state) => {
                let state = state.to_tokens();
                quote! {Action::SHIFT(Rc::new(#state))}
            }
            Action::REDUCE(production) => {
                let production = production.to_tokens();
                quote! {Action::REDUCE(Rc::new(#production))}
            }
            Action::ERROR(err) => quote! {Action::ERROR(String::new(#err))},
            Action::ACCEPT => quote! {Action::ACCEPT},
        }
    }
}
