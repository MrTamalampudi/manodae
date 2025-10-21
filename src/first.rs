use std::{ops::Deref, rc::Rc};

use indexmap::{IndexMap, IndexSet};

use crate::{grammar::Grammar, symbol::Symbol};

pub fn compute_first_set<AST, Token, TranslatorStack>(
    grammar: &Grammar<AST, Token, TranslatorStack>,
) -> IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> {
    let mut first_map: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> = IndexMap::new();
    let mut productions_hashmap: IndexMap<Rc<Symbol>, Vec<Vec<Rc<Symbol>>>> = IndexMap::new();

    grammar.terminals.iter().for_each(|terminal| {
        first_map.insert(Rc::clone(terminal), IndexSet::from([Rc::clone(terminal)]));
    });

    grammar.non_terminals.iter().for_each(|non_terminal| {
        first_map.insert(Rc::clone(non_terminal), IndexSet::new());
        productions_hashmap.insert(Rc::clone(non_terminal), vec![]);
    });

    for production in grammar.productions.iter() {
        productions_hashmap
            .entry(Rc::new(Symbol::NONTERMINAL(production.head.clone())))
            .and_modify(|entry| entry.push(production.body.clone()));
    }

    let mut changed = true;
    while changed {
        changed = false;
        for non_terminal in grammar.non_terminals.iter() {
            let productions = productions_hashmap.get_mut(non_terminal).unwrap();
            productions.iter().for_each(|body| {
                if !body.is_empty() {
                    let first = body.first().unwrap();
                    let set_ = match first.deref() {
                        Symbol::TERMINAL(_) => IndexSet::from([Rc::clone(first)]),
                        Symbol::NONTERMINAL(_) => {
                            let a = first_map.get(first).unwrap().iter().cloned().collect();
                            a
                        }
                    };
                    if !set_.is_empty() {
                        changed = true
                    }
                    let set = first_map.get_mut(non_terminal).unwrap();
                    set.extend(set_);
                }
            });
        }
    }
    first_map
}
