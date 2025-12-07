use std::{ops::Deref, os::unix::net::Incoming};

use quote::{quote, ToTokens};

use crate::state::State;

impl<AST, Token, TranslatorStack> ToTokens for State<AST, Token, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let outgoing = quote! {IndexMap::new()};
        let incoming = quote! {Vec::new()};
        let transition_productions = &self.transition_productions;

        let state = quote! {
            State {
                index:#self.index,
                items:#self.items,
                transition_productions: vec![#(#transition_productions),*],
                outgoing: #outgoing,
                incoming: #incoming
            }
        };
        tokens.extend(state);
    }
}
