use quote::{quote, ToTokens};

use crate::parser::LR1_Parser;

impl<AST, Token, TranslatorStack> ToTokens for LR1_Parser<AST, Token, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let LR1_automata = self.LR1_automata.clone();

        let follow_set: Vec<_> = self
            .follow_set
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value.iter().collect();
                let value = quote! {IndexSet::from([#(Rc::new(#value)),*])};
                quote! {(Rc::new(#key),#value)}
            })
            .collect();

        let first_set: Vec<_> = self
            .first_set
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value.iter().collect();
                let value = quote! {IndexSet::from([#(Rc::new(#value)),*])};
                quote! {(Rc::new(#key),#value)}
            })
            .collect();

        let goto: Vec<_> = self
            .goto
            .iter()
            .map(|(key, value)| {
                let value: Vec<_> = value
                    .iter()
                    .map(|(key, value)| quote! {(Rc::new(#key),Rc::new(#value))})
                    .collect();
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
                    .map(|(key, value)| quote! {(Rc::new(#key),#value)})
                    .collect();
                let value = quote! {IndexMap::from([#(#value),*])};
                quote! {(Rc::new(#key),#value)}
            })
            .collect();
        let parser = quote! {
            Parser {
                grammar: #self.grammar,
                LR1_automata: vec![#(Rc::new(#LR1_automata)),*],
                follow_set: IndexMap::from([#(#follow_set),*]),
                first_set: IndexMap::from([#(#first_set),*]),
                conflicts: false,
                goto: IndexMap::from([#(#goto),*]),
                action: IndexMap::from([#(#action),*]),,
                item_closure_map:IndexMap::new(),
                closure_map:IndexMap::new(),
            }
        };
        tokens.extend(parser);
    }
}
