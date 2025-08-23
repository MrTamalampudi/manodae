use std::{collections::HashMap, fmt::Debug, hash::Hash, ptr};

use indexmap::IndexMap;

use crate::{production::Production, symbol::Symbol};

#[derive(Debug, Clone, PartialEq)]
pub struct Item<'a, AST, Token, TranslatorStack> {
    pub production: &'a Production<AST, Token, TranslatorStack>,
    pub cursor: u8,
    pub lookaheads: Vec<Symbol>,
}

impl<'a, AST, Token, TranslatorStack> Item<'a, AST, Token, TranslatorStack> {
    pub fn next_symbol(&self) -> Option<&Symbol> {
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
        self.production.head == String::from("S'")
    }

    pub fn is_core_eq(&self, other: &Item<AST, Token, TranslatorStack>) -> bool {
        ptr::eq(self.production, other.production) && self.cursor == other.cursor
    }
}

struct ItemProduction<'a, AST, Token, TranslatorStack> {
    production: &'a Production<AST, Token, TranslatorStack>,
    cursor: u8,
}

impl<'a, AST, Token, TranslatorStack> PartialEq
    for ItemProduction<'a, AST, Token, TranslatorStack>
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.production, other.production) && self.cursor == other.cursor
    }
}

impl<'a, AST, Token, TranslatorStack> Eq for ItemProduction<'a, AST, Token, TranslatorStack> {}

impl<'a, AST, Token, TranslatorStack> Hash for ItemProduction<'a, AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.production, state);
        self.cursor.hash(state);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Items<'a, AST, Token, TranslatorStack>(pub Vec<Item<'a, AST, Token, TranslatorStack>>);

impl<'a, AST, Token, TranslatorStack> Items<'a, AST, Token, TranslatorStack>
where
    AST: PartialEq + Clone + Debug,
    Token: PartialEq + Clone + Debug,
    TranslatorStack: PartialEq + Clone + Debug,
{
    pub fn merge_cores(&mut self) {
        let mut new_items: IndexMap<
            ItemProduction<AST, Token, TranslatorStack>,
            Item<AST, Token, TranslatorStack>,
        > = IndexMap::new();
        for item in self.0.iter() {
            let item_production = ItemProduction {
                production: item.production,
                cursor: item.cursor,
            };
            new_items
                .entry(item_production)
                .and_modify(|new_item: &mut Item<'a, AST, Token, TranslatorStack>| {
                    for la in item.lookaheads.iter() {
                        if !new_item.lookaheads.contains(la) {
                            new_item.lookaheads.push(la.clone());
                        }
                    }
                })
                .or_insert(item.clone());
        }
        self.0.clear();
        self.0.extend(new_items.into_values().collect::<Vec<_>>());
    }
}
