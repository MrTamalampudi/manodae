use proc_macro2::TokenStream;
use quote::quote;

use crate::{action::Action, codegen::ToTokens};

impl ToTokens for Action {
    fn to_tokens(&self) -> TokenStream {
        match self {
            Action::SHIFT(state) => {
                let state = state.to_tokens();
                quote! {A::SHIFT(#state)}
            }
            Action::REDUCE(production) => {
                let production = production.to_tokens();
                quote! {A::REDUCE(#production)}
            }
            Action::ERROR(err) => quote! {A::ERROR(String::new(#err))},
            Action::ACCEPT => quote! {A::ACCEPT},
        }
    }
}
