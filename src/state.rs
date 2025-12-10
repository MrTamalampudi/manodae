use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{self, Hash},
    ops::Deref,
    rc::Rc,
};

use indexmap::IndexMap;

use crate::{
    interner::Interner,
    item::{Item, ItemVecExtension},
    production::Productions,
    symbol::{SymbolId, AUGMENT_START_SYMBOL_ID},
};

#[derive(Debug, Clone)]
pub struct State {
    pub index: usize,
    pub items: Vec<Item>,
    pub transition_symbol: SymbolId,
    pub outgoing: IndexMap<SymbolId, Rc<RefCell<State>>>,
    pub incoming: Vec<Rc<RefCell<State>>>,
}

//Uniquely identifies a state by its item.cursor and item.production fields
//to merge sets we donot care lookaheads if productions and cursors are same
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
    pub fn new(index: usize, items: Vec<Item>, transition_symbol: SymbolId) -> State {
        State {
            index,
            items,
            transition_symbol,
            outgoing: IndexMap::new(),
            incoming: vec![],
        }
    }

    pub fn transistion_productions<'a, AST, Token, TranslatorStack>(
        &'a self,
        productions: &Productions<AST, Token, TranslatorStack>,
    ) -> Vec<&'a Item>
    where
        AST: Clone,
        Token: Clone,
        TranslatorStack: Clone,
    {
        let transistion_symbol = self.transition_symbol;
        if self.transition_symbol == AUGMENT_START_SYMBOL_ID {
            return vec![];
        }
        self.items
            .iter()
            .filter(|item| {
                if item.cursor == 0 {
                    return false;
                }
                let production = productions.lookup(item.production);
                if production.body[(item.cursor - 1) as usize] == transistion_symbol {
                    return true;
                } else {
                    return false;
                }
            })
            .collect()
    }
}

pub trait StateVecExtension<T> {
    fn merge_sets(&mut self);
    fn custom_get(&self, state: &Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>>;
    fn custom_contains(&self, other: &Rc<RefCell<T>>) -> bool;
}

impl StateVecExtension<State> for Vec<Rc<RefCell<State>>> {
    fn merge_sets(&mut self) {
        let mut new_states: IndexMap<State, Rc<RefCell<State>>> = IndexMap::new();
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
                && state.borrow().transition_symbol == other.borrow().transition_symbol
                && state.borrow().index == other.borrow().index
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct StateId(pub usize);

#[derive(Debug, Clone)]
pub struct States {
    pub map: IndexMap<State, StateId>,
    pub vec: Vec<State>,
}

impl States {
    pub fn new() -> States {
        States {
            map: IndexMap::new(),
            vec: vec![],
        }
    }
}

impl Interner for States {
    type Id = StateId;
    type T = State;

    fn intern(&mut self, state: Self::T) -> Self::Id {
        if let Some(&id) = self.map.get(&state) {
            return id;
        }

        let id = StateId(self.map.len());
        self.map.insert(state.clone(), id);
        self.vec.push(state);

        id
    }

    fn lookup(&self, id: Self::Id) -> Self::T {
        self.vec[id.0].clone()
    }

    fn reverse_lookup(&self, production: &Self::T) -> Option<Self::Id> {
        self.map.get(production).map(|x| *x)
    }
}
