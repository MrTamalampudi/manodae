use quote::quote;

use crate::{codegen::ToTokens, state::State};

impl<AST, Token, TranslatorStack> ToTokens for State<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let outgoing = quote! {IndexMap::new()};
        let incoming = quote! {Vec::new()};
        let transition_productions: Vec<_> =
            self.items.iter().map(|item| item.to_tokens()).collect();
        let index = &self.index;
        let items: Vec<_> = self.items.iter().map(|item| item.to_tokens()).collect();

        let state = quote! {
            State {
                index:#index,
                items:vec![#(#items),*],
                transition_productions: vec![#(#transition_productions),*],
                outgoing: #outgoing,
                incoming: #incoming
            }
        };
        state
    }
}
