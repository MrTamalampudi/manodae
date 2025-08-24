use std::{hash::Hash, sync::Arc};

use crate::{error::ParseError, symbol::Symbol};

#[derive(Clone)]
pub struct Production<AST, Token, TranslatorStack> {
    pub head: String,
    pub body: Vec<Symbol>,
    pub error_message: Option<String>,
    pub action: Option<
        Arc<
            dyn Fn(
                &mut AST,
                &mut Vec<Token>,
                &mut Vec<TranslatorStack>,
                &mut Vec<ParseError<Token>>,
            ),
        >,
    >,
}

impl<AST, Token, TranslatorStack> Hash for Production<AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
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
            .finish_non_exhaustive()
    }
}

impl<AST, Token, TranslatorStack> PartialEq for Production<AST, Token, TranslatorStack> {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head
            && self.body == other.body
            && self.error_message == other.error_message
    }
}

impl<AST, Token, TranslatorStack> Eq for Production<AST, Token, TranslatorStack> {}

impl<AST, Token, TranslatorStack> Production<AST, Token, TranslatorStack> {
    pub fn is_augmented_production(&self) -> bool {
        self.head == String::from("S'")
    }
}
