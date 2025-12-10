use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::ToTokens,
    state::{State, StateId, States},
};

impl ToTokens for State {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let transition_symbol = self.transition_symbol.to_tokens();
        let index = &self.index;
        let items: Vec<_> = self.items.iter().map(|item| item.to_tokens()).collect();

        let state = quote! {
            S::new(#index,vec![#(#items),*],#transition_symbol)
        };
        state
    }
}

impl ToTokens for StateId {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let id = self.0;
        quote! {PID(#id)}
    }
}

impl ToTokens for States {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let map: Vec<TokenStream> = self
            .map
            .iter()
            .map(|(_, value)| {
                let index = value.0;
                let key = quote! { sc!(#index)};
                let value = value.to_tokens();
                quote! {(#key,#value)}
            })
            .collect();
        let vec_: Vec<TokenStream> = self.vec.iter().map(|symbol| symbol.to_tokens()).collect();
        let states = quote! {
            let states = vec![#(#vec_),*];
            States {
                map: IndexMap::from([#(#map),*]),
                vec: states,
            }
        };
        states
    }
}
