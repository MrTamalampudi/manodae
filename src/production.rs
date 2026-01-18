use std::{hash::Hash, rc::Rc};

use indexmap::IndexMap;
use logos::Span;
use proc_macro2::TokenStream;

use crate::{
    error::ParseError,
    interner::Interner,
    symbol::{SymbolId, AUGMENT_START_SYMBOL_ID, START_SYMBOL_ID},
};

///A production is uniquely identified by its head,body,error_message,index
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
                &mut Vec<(Token, Span)>,
                &mut Vec<TranslatorStack>,
                &mut Vec<ParseError>,
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
                    &mut Vec<(Token, Span)>,
                    &mut Vec<TranslatorStack>,
                    &mut Vec<ParseError>,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ProductionId(pub usize);

pub(crate) const AUGMENT_PRODUCTION_ID: ProductionId = ProductionId(0);

#[derive(Debug, Clone)]
pub struct Productions<AST, Token, TranslatorStack> {
    pub map: IndexMap<Production<AST, Token, TranslatorStack>, ProductionId>,
    pub vec: Vec<Production<AST, Token, TranslatorStack>>,
}

impl<AST, Token, TranslatorStack> Hash for Productions<AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

impl<AST, Token, TranslatorStack> Productions<AST, Token, TranslatorStack>
where
    AST: Clone,
    Token: Clone,
    TranslatorStack: Clone,
{
    pub fn new() -> Self {
        let mut productions = Productions {
            map: IndexMap::new(),
            vec: vec![],
        };
        let augmented_production = Production {
            head: AUGMENT_START_SYMBOL_ID,
            body: vec![START_SYMBOL_ID],
            error_message: None,
            #[allow(unused_variables)]
            action: Some(Rc::new(|ast, token_stack, tl_stack, errors| {})),
            action_tokens: quote::quote! {Some(Rc::new(|ast, token_stack, tl_stack, errors| {}))},
            index: 0,
        };
        productions.intern(augmented_production);
        productions
    }
}

impl<AST, Token, TranslatorStack> Interner for Productions<AST, Token, TranslatorStack>
where
    AST: Clone,
    Token: Clone,
    TranslatorStack: Clone,
{
    type T = Production<AST, Token, TranslatorStack>;
    type Id = ProductionId;
    fn intern(&mut self, production: Self::T) -> Self::Id {
        if let Some(&id) = self.map.get(&production) {
            return id;
        }
        let id = ProductionId(self.map.len());
        self.map.insert(production.clone(), id);
        self.vec.push(production);

        id
    }

    fn lookup(&self, id: Self::Id) -> Self::T {
        self.vec[id.0].clone()
    }

    fn reverse_lookup(&self, production: &Self::T) -> Option<Self::Id> {
        self.map.get(production).map(|x| *x)
    }
}
