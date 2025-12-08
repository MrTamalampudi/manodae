use proc_macro2::TokenStream;
use quote::quote;

use crate::{codegen::ToTokens, parser::LR1_Parser};

impl<AST, Token, TranslatorStack> ToTokens for LR1_Parser<AST, Token, TranslatorStack> {
    fn to_tokens(&self) -> TokenStream {
        let LR1_automata: Vec<_> = self
            .LR1_automata
            .iter()
            .map(|state| {
                let state = state.to_tokens();
                quote! {Rc::new(#state)}
            })
            .collect();

        let follow_set: Vec<_> = self
            .follow_set
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value
                    .iter()
                    .map(|sym| {
                        let tokens = sym.to_tokens();
                        quote! {Rc::new(#tokens)}
                    })
                    .collect();
                let value = quote! {IndexSet::from([#(#value),*])};
                let key = key.to_tokens();
                quote! {(Rc::new(#key),#value)}
            })
            .collect();

        let first_set: Vec<_> = self
            .first_set
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value
                    .iter()
                    .map(|sym| {
                        let tokens = sym.to_tokens();
                        quote! {Rc::new(#tokens)}
                    })
                    .collect();
                let value = quote! {IndexSet::from([#(#value),*])};
                let key = key.to_tokens();
                quote! {(Rc::new(#key),#value)}
            })
            .collect();

        let goto: Vec<_> = self
            .goto
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value
                    .iter()
                    .map(|(key, value)| {
                        let key = key.to_tokens();
                        let value = value.to_tokens();
                        quote! {(Rc::new(#key),Rc::new(#value))}
                    })
                    .collect();
                let key = key.to_tokens();
                let value = quote! {IndexMap::from([#(#value),*])};
                quote! {(Rc::new(#key),#value)}
            })
            .collect();

        let action: Vec<_> = self
            .action
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value
                    .iter()
                    .map(|(key, value)| {
                        let key = key.to_tokens();
                        let value = value.to_tokens();
                        quote! {(Rc::new(#key),#value)}
                    })
                    .collect();
                let key = key.to_tokens();
                let value = quote! {IndexMap::from([#(#value),*])};
                quote! {(Rc::new(#key),#value)}
            })
            .collect();
        let grammar = self.grammar.to_tokens();
        let parser = quote! {
            Parser {
                grammar: #grammar,
                LR1_automata: vec![#(#LR1_automata),*],
                follow_set: IndexMap::from([#(#follow_set),*]),
                first_set: IndexMap::from([#(#first_set),*]),
                conflicts: false,
                goto: IndexMap::from([#(#goto),*]),
                action: IndexMap::from([#(#action),*]),,
                item_closure_map:IndexMap::new(),
                closure_map:IndexMap::new(),
            }
        };
        parser
    }
}
