use proc_macro2::TokenStream;
use quote::quote;

use crate::{action::Action, codegen::ToTokens};

impl ToTokens for Action {
    fn to_tokens(&self) -> TokenStream {
        match self {
            Action::SHIFT(state) => {
                let state = state.to_tokens();
                //expands to ACTION::SHIFT(state)
                quote! {S(#state)}
            }
            Action::REDUCE(production) => {
                let production = production.to_tokens();
                //expands to ACTION::REDUCE(production)
                quote! {R(#production)}
            }
            //expands to ACTION::ERROR(String)
            Action::ERROR(err) => quote! {E(String::new(#err))},
            //expands to ACTION::ACCEPT
            Action::ACCEPT => quote! {A},
        }
    }
}
