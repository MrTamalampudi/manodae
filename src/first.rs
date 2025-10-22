use std::{ops::Deref, rc::Rc};

use indexmap::{IndexMap, IndexSet};

use crate::{grammar::Grammar, symbol::Symbol};

pub fn compute_first_set<AST, Token, TranslatorStack>(
    productions: &Grammar<AST, Token, TranslatorStack>,
) -> IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> {
    let mut first_map: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> = IndexMap::new();
    let mut first_map_: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> = IndexMap::new();
    let mut productions_hashmap: IndexMap<Rc<Symbol>, Vec<Vec<Rc<Symbol>>>> = IndexMap::new();

    let mut symbols = vec![Rc::new(Symbol::TERMINAL(String::from("EOF")))];
    symbols.extend(productions.non_terminals.clone());
    symbols.extend(productions.terminals.clone());

    symbols.iter().for_each(|symbol| {
        first_map.insert(symbol.clone(), IndexSet::new());
        if let Symbol::NONTERMINAL(_) = symbol.deref() {
            productions_hashmap.insert(symbol.clone(), Vec::new());
        }
    });

    for production in productions.productions.iter() {
        let p = productions_hashmap.get_mut(&Symbol::NONTERMINAL(production.head.clone()));
        if let Some(bodies) = p {
            bodies.push(production.body.clone());
        }
    }

    while first_map_.ne(&first_map) {
        first_map_.clear();
        first_map_ = first_map.clone();
        for symbol in symbols.iter() {
            match symbol.deref() {
                Symbol::TERMINAL(_) => {
                    first_map.insert(symbol.clone(), IndexSet::from([symbol.clone()]));
                }
                Symbol::NONTERMINAL(_) => {
                    let p = productions_hashmap.get(symbol).unwrap();
                    p.iter().for_each(|body| {
                        if !body.is_empty() {
                            let first = body.first().unwrap();
                            let set_ = match first.deref() {
                                Symbol::TERMINAL(_) => IndexSet::from([first.clone()]),
                                Symbol::NONTERMINAL(_) => {
                                    let a: IndexSet<Rc<Symbol>> =
                                        first_map.get(first).unwrap().iter().cloned().collect();
                                    a
                                }
                            };
                            let set = first_map.get_mut(symbol).unwrap();
                            set.extend(set_);
                        }
                    });
                }
            }
        }
    }
    first_map
}
