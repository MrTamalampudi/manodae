use quote::{quote, ToTokens};

use crate::action::Action;

impl<AST, Token, TranslatorStack> ToTokens for Action<AST, Token, TranslatorStack> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let t = match self {
            Action::SHIFT(state) => quote! {Action::SHIFT(Rc::new(#state))},
            Action::REDUCE(production) => quote! {Action::REDUCE(Rc::new(#production))},
            Action::ERROR(err) => quote! {Action::ERROR(String::new(#err))},
            Action::ACCEPT => quote! {Action::ACCEPT},
        };
        tokens.extend(t);
    }
}
