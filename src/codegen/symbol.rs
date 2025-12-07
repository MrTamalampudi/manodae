use quote::{quote, ToTokens};

use crate::symbol::Symbol;

impl ToTokens for Symbol {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let t = match self {
            Symbol::TERMINAL(terminal) => quote! { Symbol::TERMINAL(String::from(#terminal))},
            Symbol::NONTERMINAL(terminal) => quote! { Symbol::NONTERMINAL(String::from(#terminal))},
        };
        tokens.extend(t);
    }
}
