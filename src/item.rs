use std::{fmt::Debug, ptr};

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

#[derive(Debug, PartialEq, Clone)]
pub struct Items<'a, AST, Token, TranslatorStack>(pub Vec<Item<'a, AST, Token, TranslatorStack>>);

impl<'a, AST, Token, TranslatorStack> Items<'a, AST, Token, TranslatorStack>
where
    AST: PartialEq + Clone + Debug,
    Token: PartialEq + Clone + Debug,
    TranslatorStack: PartialEq + Clone + Debug,
{
    //Room for optimisation is there here
    pub fn merge_cores(&mut self) {
        let mut new_items: Vec<Item<'a, AST, Token, TranslatorStack>> = vec![];
        let contain_item_with_core =
            |items: &Vec<Item<'a, AST, Token, TranslatorStack>>,
             item: &Item<AST, Token, TranslatorStack>| {
                items.iter().any(|it| it.is_core_eq(item))
            };
        for item in self.0.iter() {
            if !contain_item_with_core(&new_items, item) {
                new_items.push(item.clone());
            } else {
                let new_item = new_items
                    .iter_mut()
                    .find(|ni| item.is_core_eq(*ni))
                    .unwrap();
                for la in item.lookaheads.iter() {
                    if !new_item.lookaheads.contains(la) {
                        new_item.lookaheads.push(la.clone());
                    }
                }
            }
        }
        self.0.clear();
        self.0.extend(new_items);
    }
}
