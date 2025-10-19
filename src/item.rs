use std::{fmt::Debug, hash::Hash, ptr};

use indexmap::IndexMap;

use crate::{production::Production, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct Item<'a, AST, Token, TranslatorStack> {
    pub production: &'a Production<AST, Token, TranslatorStack>,
    pub cursor: u8,
    pub lookaheads: Vec<Symbol>,
}

impl<'a, AST, Token, TranslatorStack> PartialEq for Item<'a, AST, Token, TranslatorStack> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.production, other.production) && self.cursor == other.cursor
    }
}

impl<'a, AST, Token, TranslatorStack> Eq for Item<'a, AST, Token, TranslatorStack> {}

impl<'a, AST, Token, TranslatorStack> Hash for Item<'a, AST, Token, TranslatorStack> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.production, state);
        self.cursor.hash(state);
    }
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

    pub fn is_eq(&self, other: &Item<AST, Token, TranslatorStack>) -> bool {
        ptr::eq(self.production, other.production)
            && self.cursor == other.cursor
            && self.lookaheads == other.lookaheads
    }
}

pub trait ItemVecExtension<T> {
    fn merge_cores(&mut self);
    fn custom_contains(&self, item_to_find: &T) -> bool;
}

impl<'a, AST, Token, TranslatorStack> ItemVecExtension<Item<'a, AST, Token, TranslatorStack>>
    for Vec<Item<'a, AST, Token, TranslatorStack>>
where
    AST: PartialEq + Clone + Debug,
    Token: PartialEq + Clone + Debug,
    TranslatorStack: PartialEq + Clone + Debug,
{
    fn merge_cores(&mut self) {
        let mut new_items: IndexMap<
            Item<AST, Token, TranslatorStack>,
            Item<AST, Token, TranslatorStack>,
        > = IndexMap::new();
        for item in self.iter() {
            new_items
                .entry(item.clone())
                .and_modify(|new_item: &mut Item<'a, AST, Token, TranslatorStack>| {
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

    fn custom_contains(&self, other: &Item<'a, AST, Token, TranslatorStack>) -> bool {
        self.iter().any(|item| {
            ptr::eq(item.production, other.production)
                && item.cursor == other.cursor
                && item.lookaheads == other.lookaheads
        })
    }
}
