use std::fmt::Debug;

use indexmap::IndexMap;

use crate::item::{Item, ItemProduction, Items};

#[derive(Debug, Clone, PartialEq)]
pub struct State<'a, AST, Token, TranslatorStack> {
    pub index: usize,
    pub items: Vec<Item<'a, AST, Token, TranslatorStack>>,
    pub transition_productions: Vec<Item<'a, AST, Token, TranslatorStack>>,
}

impl<'a, AST, Token, TranslatorStack> State<'a, AST, Token, TranslatorStack>
where
    AST: Clone,
    Token: Clone,
    TranslatorStack: Clone,
{
    pub fn new(
        index: usize,
        items: Vec<Item<'a, AST, Token, TranslatorStack>>,
        transition_productions: Vec<Item<'a, AST, Token, TranslatorStack>>,
    ) -> State<'a, AST, Token, TranslatorStack> {
        let state = State {
            index,
            items,
            transition_productions,
        };
        state
    }

    pub fn merge_cores(&mut self) {
        let mut new_items: IndexMap<
            ItemProduction<AST, Token, TranslatorStack>,
            Item<AST, Token, TranslatorStack>,
        > = IndexMap::new();
        for item in self.items.iter() {
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
        self.items.clear();
        self.items
            .extend(new_items.into_values().collect::<Vec<_>>());
    }
}
