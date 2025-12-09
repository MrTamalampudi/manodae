use std::{fmt::Debug, hash::Hash};

use indexmap::IndexMap;

use crate::{
    interner::Interner,
    production::{ProductionId, Productions},
    symbol::SymbolId,
};

#[derive(Debug, Clone)]
pub struct Item {
    pub production: ProductionId,
    pub cursor: u8,
    pub lookaheads: Vec<SymbolId>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.production.eq(&other.production)
            && self.cursor == other.cursor
            && self.lookaheads == other.lookaheads
    }
}

impl Eq for Item {}

impl Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.production.hash(state);
        self.cursor.hash(state);
        self.lookaheads.hash(state);
    }
}

impl Item {
    pub fn n(production: ProductionId, cursor: u8, lookaheads: Vec<SymbolId>) -> Self {
        Item {
            production,
            cursor,
            lookaheads,
        }
    }
    pub fn next_symbol<AST, Token, TranslatorStack>(
        &self,
        productions: &Productions<AST, Token, TranslatorStack>,
    ) -> Option<SymbolId>
    where
        AST: Clone,
        Token: Clone,
        TranslatorStack: Clone,
    {
        let production = productions.lookup(self.production);
        if self.cursor == production.body.len() as u8 {
            None
        } else {
            let symbolId = production.body.get(self.cursor as usize).unwrap();
            Some(*symbolId)
        }
    }
    pub fn advance_cursor(&mut self) {
        self.cursor += 1;
    }

    pub fn is_eq(&self, other: &Item) -> bool {
        self.production.eq(&other.production)
            && self.cursor == other.cursor
            && self.lookaheads == other.lookaheads
    }
}

pub trait ItemVecExtension<T> {
    fn merge_cores(&mut self);
}

impl ItemVecExtension<Item> for Vec<Item> {
    fn merge_cores(&mut self) {
        let mut new_items: IndexMap<(ProductionId, u8), Item> = IndexMap::new();
        for item in self.iter() {
            new_items
                .entry((item.production, item.cursor))
                .and_modify(|new_item: &mut Item| {
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
