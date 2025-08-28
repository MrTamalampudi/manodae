use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{self, Hash},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use indexmap::IndexMap;

use crate::{
    item::{Item, ItemVecExtension},
    traits::VecExtension,
};

#[derive(Debug, Clone)]
pub struct State<'a, AST, Token, TranslatorStack> {
    pub index: usize,
    pub items: Vec<Item<'a, AST, Token, TranslatorStack>>,
    pub transition_productions: Vec<Item<'a, AST, Token, TranslatorStack>>,
}

impl<'a, AST, Token, TranslatorStack> Hash for State<'a, AST, Token, TranslatorStack> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.items.hash(state);
        self.transition_productions.hash(state);
    }
}

impl<'a, AST, Token, TranslatorStack> Eq for State<'a, AST, Token, TranslatorStack> {}

impl<'a, AST, Token, TranslatorStack> PartialEq for State<'a, AST, Token, TranslatorStack> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items && self.transition_productions == other.transition_productions
    }
}

impl<'a, AST, Token, TranslatorStack> VecExtension<State<'a, AST, Token, TranslatorStack>>
    for Vec<State<'a, AST, Token, TranslatorStack>>
{
    fn custom_contains(&self, other: &State<'a, AST, Token, TranslatorStack>) -> bool {
        self.iter().any(|state| {
            state
                .items
                .iter()
                .any(|item| other.items.custom_contains(item))
                && state.transition_productions == other.transition_productions
                && state.index == other.index
        })
    }
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
        State {
            index,
            items,
            transition_productions,
        }
    }
}

pub trait StateVecExtension {
    fn merge_sets(&mut self);
}

impl<'a, AST, Token, TranslatorStack> StateVecExtension
    for Vec<Rc<State<'a, AST, Token, TranslatorStack>>>
where
    AST: Clone + PartialEq + Debug,
    Token: Clone + PartialEq + Debug,
    TranslatorStack: Clone + PartialEq + Debug,
{
    fn merge_sets(&mut self) {
        let mut new_states: IndexMap<
            Rc<State<'a, AST, Token, TranslatorStack>>,
            Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>,
        > = IndexMap::new();
        for state in self.iter() {
            let state_entry = new_states
                .entry(Rc::new(state.deref().clone()))
                .and_modify(|entry| {
                    entry
                        .borrow_mut()
                        .deref_mut()
                        .items
                        .extend(state.items.clone());
                    entry
                        .borrow_mut()
                        .transition_productions
                        .extend(state.transition_productions.clone());
                })
                .or_insert(Rc::new(RefCell::new(state.deref().clone())));
            state_entry.borrow_mut().items.merge_cores();
            state_entry
                .borrow_mut()
                .transition_productions
                .merge_cores();
        }
        let a = new_states
            .into_values()
            .map(|state| Rc::new(state.borrow().deref().clone()))
            .collect::<Vec<_>>();
        self.clear();
        self.extend(a);
    }
}
