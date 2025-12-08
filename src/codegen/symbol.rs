use quote::quote;

use crate::{codegen::ToTokens, symbol::Symbol};

impl ToTokens for Symbol {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let t = match self {
            Symbol::TERMINAL(terminal) => quote! { C::TERMINAL(String::from(#terminal))},
            Symbol::NONTERMINAL(terminal) => quote! { C::NONTERMINAL(String::from(#terminal))},
        };
        t
    }
}
