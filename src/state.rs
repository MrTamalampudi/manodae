use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{self, Hash},
    ops::Deref,
    rc::Rc,
};

use indexmap::IndexMap;

use crate::{
    item::{Item, ItemVecExtension},
    symbol::Symbol,
};

#[derive(Debug, Clone)]
pub struct State<'a, AST, Token, TranslatorStack> {
    pub index: usize,
    pub items: Vec<Item<'a, AST, Token, TranslatorStack>>,
    pub transition_productions: Vec<Item<'a, AST, Token, TranslatorStack>>,
    pub outgoing: IndexMap<Symbol, Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>>,
    pub incoming: Vec<Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>>,
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
        outgoing: IndexMap<Symbol, Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>>,
        incoming: Vec<Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>>,
    ) -> State<'a, AST, Token, TranslatorStack> {
        State {
            index,
            items,
            transition_productions,
            outgoing,
            incoming,
        }
    }
}

pub trait StateVecExtension<T> {
    fn merge_sets(&mut self);
    fn custom_get(&self, state: &Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>>;
    fn custom_contains(&self, other: &Rc<RefCell<T>>) -> bool;
}

impl<'a, AST, Token, TranslatorStack> StateVecExtension<State<'a, AST, Token, TranslatorStack>>
    for Vec<Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>>
where
    AST: Clone + PartialEq + Debug,
    Token: Clone + PartialEq + Debug,
    TranslatorStack: Clone + PartialEq + Debug,
{
    fn merge_sets(&mut self) {
        let mut new_states: IndexMap<
            State<'a, AST, Token, TranslatorStack>,
            Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>,
        > = IndexMap::new();
        for state in self.iter() {
            let state_entry = new_states
                .entry(state.borrow().deref().clone())
                .and_modify(|entry| {
                    let mut borrow_mut = entry.borrow_mut();
                    if !borrow_mut.incoming.contains(state) {
                        borrow_mut.incoming.extend(vec![state.clone()]);
                    }
                    borrow_mut.outgoing.extend(state.borrow().outgoing.clone());
                    borrow_mut.items.extend(state.borrow().items.clone());
                    borrow_mut
                        .transition_productions
                        .extend(state.borrow().transition_productions.clone());
                })
                .or_insert(Rc::clone(state));

            {
                let incoming_state = &state_entry.borrow().incoming;
                for i_state in incoming_state.iter().cloned() {
                    let mut outgoing_map = IndexMap::new();
                    {
                        let i_state_ = i_state.borrow();
                        {
                            for (o_symbol, o_state) in i_state_.outgoing.iter() {
                                if o_state.borrow().deref().eq(state.borrow().deref()) {
                                    outgoing_map.insert(o_symbol.clone(), Rc::clone(state_entry));
                                }
                            }
                        }
                    }
                    let mut _i_state_ = i_state.borrow_mut();
                    _i_state_.outgoing.extend(outgoing_map);
                }
            }
            state_entry.borrow_mut().items.merge_cores();
            state_entry
                .borrow_mut()
                .transition_productions
                .merge_cores();
        }
        let a = new_states.into_values().collect::<Vec<_>>();
        self.clear();
        self.extend(a);
    }

    fn custom_get(
        &self,
        state: &Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>,
    ) -> Option<Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>> {
        self.iter()
            .cloned()
            .find(|state_ref| state_ref.borrow().clone().eq(&state.borrow().clone()))
    }

    fn custom_contains(&self, other: &Rc<RefCell<State<'a, AST, Token, TranslatorStack>>>) -> bool {
        self.iter().any(|state| {
            state
                .borrow()
                .items
                .iter()
                .any(|item| other.borrow().items.custom_contains(item))
                && state.borrow().transition_productions == other.borrow().transition_productions
                && state.borrow().index == other.borrow().index
        })
    }
}
