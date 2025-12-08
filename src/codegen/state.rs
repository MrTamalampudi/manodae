use quote::quote;

use crate::{codegen::ToTokens, state::State};

impl<AST, Token, TranslatorStack> ToTokens for State<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let transition_productions: Vec<_> =
            self.items.iter().map(|item| item.to_tokens()).collect();
        let index = &self.index;
        let items: Vec<_> = self.items.iter().map(|item| item.to_tokens()).collect();

        let state = quote! {
            S::new(#index,vec![#(#items),*],vec![#(#transition_productions),*])
        };
        state
    }
}
