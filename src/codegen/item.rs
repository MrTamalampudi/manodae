use quote::{quote, ToTokens};

use crate::item::Item;

impl<AST, Token, TranslatorStack> ToTokens for Item<AST, Token, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let la = self.lookaheads.clone();
        let item = quote! {
            Item {
                production: Rc::new(#self.production),
                cursor: #self.cursor,
                lookaheads: vec![#(Rc::new(#la)),*]
            }
        };
        tokens.extend(item);
    }
}
