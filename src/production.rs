use std::{hash::Hash, rc::Rc};

use proc_macro2::TokenStream;

use crate::{
    error::ParseError,
    symbol::{SymbolId, AUGMENT_START_SYMBOL_ID},
};

#[derive(Clone)]
pub struct Production<AST, Token, TranslatorStack> {
    pub index: usize,
    pub head: SymbolId,
    pub body: Vec<SymbolId>,
    pub error_message: Option<String>,
    pub action_tokens: TokenStream,
    pub action: Option<
        Rc<
            dyn Fn(
                &mut AST,
                &mut Vec<Token>,
                &mut Vec<TranslatorStack>,
                &mut Vec<ParseError<Token>>,
            ),
        >,
    >,
}

impl<AST, Token, TranslatorStack> Production<AST, Token, TranslatorStack> {
    pub fn n(
        index: usize,
        head: SymbolId,
        body: Vec<SymbolId>,
        error_message: Option<String>,
        action_tokens: TokenStream,
        action: Option<
            Rc<
                dyn Fn(
                    &mut AST,
                    &mut Vec<Token>,
                    &mut Vec<TranslatorStack>,
                    &mut Vec<ParseError<Token>>,
                ),
            >,
        >,
    ) -> Self {
        Production {
            index,
            head,
            body,
            error_message,
            action_tokens,
            action,
        }
    }
}

impl<AST, Token, TranslatorStack> Hash for Production<AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.error_message.hash(state);
        self.head.hash(state);
        self.body.hash(state);
    }
}

impl<AST, Token, TranslatorStack> std::fmt::Debug for Production<AST, Token, TranslatorStack> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Production")
            .field("head", &self.head)
            .field("body", &self.body)
            .field("error_message", &self.error_message)
            .field("index", &self.index)
            .field("action", &self.action_tokens.to_string())
            .finish()
    }
}

impl<AST, Token, TranslatorStack> PartialEq for Production<AST, Token, TranslatorStack> {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head
            && self.body == other.body
            && self.error_message == other.error_message
            && self.index == other.index
    }
}

impl<AST, Token, TranslatorStack> Eq for Production<AST, Token, TranslatorStack> {}

impl<AST, Token, TranslatorStack> Production<AST, Token, TranslatorStack> {
    pub fn is_augmented_production(&self) -> bool {
        self.head == AUGMENT_START_SYMBOL_ID
    }

    pub fn body_len(&self) -> usize {
        self.body.len()
    }
}
