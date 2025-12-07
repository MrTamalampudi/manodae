use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::grammar::Grammar;

impl<AST, Token, TranslatorStack> ToTokens for Grammar<AST, Token, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let nt: Vec<_> = self.non_terminals.iter().collect();
        let nt = quote! {IndexSet::from([#(Rc::new(#nt)),*])};
        let t: Vec<_> = self.terminals.iter().collect();
        let t = quote! {IndexSet::from([#(Rc::new(#t)),*])};
        let start = quote! {Rc::new(#self.start)};
        let productions: Vec<_> = self.productions.iter().collect();
        let productions = quote! {IndexSet::from([#(Rc::new(#productions)),*])};
        let production_head_map = quote! {IndexMap::new()};

        let grammar = quote! {
            Grammar {
                non_terminals:#nt,
                terminals:#t,
                start:#start,
                productions:#productions,
                production_head_map:#production_head_map
            }
        };
        tokens.extend(grammar);
    }
}
