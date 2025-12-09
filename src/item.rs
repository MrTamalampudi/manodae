use std::{fmt::Debug, hash::Hash, rc::Rc};

use indexmap::IndexMap;

use crate::{
    production::Production,
    symbol::{SymbolId, AUGMENT_START_SYMBOL_ID},
};

#[derive(Debug, Clone)]
pub struct Item<AST, Token, TranslatorStack> {
    pub production: Rc<Production<AST, Token, TranslatorStack>>,
    pub cursor: u8,
    pub lookaheads: Vec<SymbolId>,
}

impl<AST, Token, TranslatorStack> PartialEq for Item<AST, Token, TranslatorStack> {
    fn eq(&self, other: &Self) -> bool {
        self.production.eq(&other.production)
            && self.cursor == other.cursor
            && self.lookaheads == other.lookaheads
    }
}

impl<AST, Token, TranslatorStack> Eq for Item<AST, Token, TranslatorStack> {}

impl<AST, Token, TranslatorStack> Hash for Item<AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.production.hash(state);
        self.cursor.hash(state);
        self.lookaheads.hash(state);
    }
}

impl<AST, Token, TranslatorStack> Item<AST, Token, TranslatorStack> {
    pub fn n(
        production: Rc<Production<AST, Token, TranslatorStack>>,
        cursor: u8,
        lookaheads: Vec<SymbolId>,
    ) -> Self {
        Item {
            production,
            cursor,
            lookaheads,
        }
    }
    pub fn next_symbol(&self) -> Option<&SymbolId> {
        if self.cursor == self.production.body.len() as u8 {
            None
        } else {
            self.production.body.get(self.cursor as usize)
        }
    }
    pub fn advance_cursor(&mut self) {
        self.cursor += 1;
    }
    pub fn is_augment_item(&self) -> bool {
        self.production.head == AUGMENT_START_SYMBOL_ID
    }

    pub fn is_eq(&self, other: &Item<AST, Token, TranslatorStack>) -> bool {
        self.production.eq(&other.production)
            && self.cursor == other.cursor
            && self.lookaheads == other.lookaheads
    }
}

pub trait ItemVecExtension<T> {
    fn merge_cores(&mut self);
}

impl<'a, AST, Token, TranslatorStack> ItemVecExtension<Item<AST, Token, TranslatorStack>>
    for Vec<Item<AST, Token, TranslatorStack>>
where
    AST: PartialEq + Clone + Debug,
    Token: PartialEq + Clone + Debug,
    TranslatorStack: PartialEq + Clone + Debug,
{
    fn merge_cores(&mut self) {
        let mut new_items: IndexMap<
            (Rc<Production<AST, Token, TranslatorStack>>, u8),
            Item<AST, Token, TranslatorStack>,
        > = IndexMap::new();
        for item in self.iter() {
            new_items
                .entry((Rc::clone(&item.production), item.cursor))
                .and_modify(|new_item: &mut Item<AST, Token, TranslatorStack>| {
                    for la in item.lookaheads.iter() {
                        if !new_item.lookaheads.contains(la) {
                            new_item.lookaheads.push(la.clone());
                        }
                    }
                })
                .or_insert(item.clone());
        }
        self.clear();
        self.extend(new_items.into_values().collect::<Vec<_>>());
    }
}
