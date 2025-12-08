use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, item::Item};

impl<AST, Token, TranslatorStack> ToTokens for Item<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let la: Vec<_> = self
            .lookaheads
            .iter()
            .map(|sym| {
                let tokens = sym.to_tokens();
                quote! {Rc::new(#tokens)}
            })
            .collect();
        let production = &self.production.to_tokens();
        let cursor = &self.cursor;
        let item = quote! {
            I::n(
                Rc::new(#production),
                #cursor,
                vec![#(#la),*]
            )
        };
        item
    }
}
