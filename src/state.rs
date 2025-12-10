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
    symbol::SymbolId,
};

#[derive(Debug, Clone)]
pub struct State {
    pub index: usize,
    pub items: Vec<Item>,
    pub transition_productions: Vec<Item>,
    pub outgoing: IndexMap<SymbolId, Rc<RefCell<State>>>,
    pub incoming: Vec<Rc<RefCell<State>>>,
}

impl Hash for State {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.items.iter().for_each(|item| {
            item.cursor.hash(state);
            item.production.hash(state);
        });
    }
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        for (item, item2) in self.items.iter().zip(other.items.iter()) {
            if item.cursor != item2.cursor || item.production != item2.production {
                return false;
            }
        }
        return true;
    }
}

impl State {
    pub fn new(index: usize, items: Vec<Item>, transition_productions: Vec<Item>) -> State {
        State {
            index,
            items,
            transition_productions,
            outgoing: IndexMap::new(),
            incoming: vec![],
        }
    }
}

struct HashByItem(State);

impl Hash for HashByItem {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.items.iter().for_each(|item| {
            item.cursor.hash(state);
            item.production.hash(state);
        });
    }
}

impl PartialEq for HashByItem {
    fn eq(&self, other: &Self) -> bool {
        for (item, item2) in self.0.items.iter().zip(other.0.items.iter()) {
            if item.cursor != item2.cursor || item.production != item2.production {
                return false;
            }
        }
        return true;
    }
}

impl Eq for HashByItem {}

pub trait StateVecExtension<T> {
    fn merge_sets(&mut self);
    fn custom_get(&self, state: &Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>>;
    fn custom_contains(&self, other: &Rc<RefCell<T>>) -> bool;
}

impl StateVecExtension<State> for Vec<Rc<RefCell<State>>> {
    fn merge_sets(&mut self) {
        let mut new_states: IndexMap<HashByItem, Rc<RefCell<State>>> = IndexMap::new();
        for state in self.iter() {
            let state_entry = new_states
                .entry(HashByItem(state.borrow().deref().clone()))
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

    fn custom_get(&self, state: &Rc<RefCell<State>>) -> Option<Rc<RefCell<State>>> {
        self.iter()
            .cloned()
            .find(|state_ref| state_ref.borrow().clone().eq(&state.borrow().clone()))
    }

    fn custom_contains(&self, other: &Rc<RefCell<State>>) -> bool {
        self.iter().any(|state| {
            state
                .borrow()
                .items
                .iter()
                .any(|item| other.borrow().items.contains(item))
                && state.borrow().transition_productions == other.borrow().transition_productions
                && state.borrow().index == other.borrow().index
        })
    }
}
