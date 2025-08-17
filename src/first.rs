use std::collections::{HashMap, HashSet};

use crate::{
    production::Production,
    symbol::{unique_symbols, Symbol},
};

pub fn compute_first_set<AST, Token, TranslatorStack>(
    productions: &Vec<Production<AST, Token, TranslatorStack>>,
) -> HashMap<Symbol, HashSet<String>> {
    let symbols = unique_symbols(productions);
    let mut first_map: HashMap<Symbol, HashSet<String>> = HashMap::new();
    let mut first_map_: HashMap<Symbol, HashSet<String>> = HashMap::new();
    let mut productions_hashmap: HashMap<Symbol, Vec<Vec<Symbol>>> = HashMap::new();

    symbols.iter().for_each(|symbol| {
        first_map.insert(symbol.clone(), HashSet::new());
        if let Symbol::NONTERMINAL(_) = symbol {
            productions_hashmap.insert(symbol.clone(), Vec::new());
        }
    });

    for production in productions.iter() {
        let p = productions_hashmap.get_mut(&Symbol::NONTERMINAL(production.head.clone()));
        if let Some(bodies) = p {
            bodies.push(production.body.clone());
        }
    }

    while first_map_.ne(&first_map) {
        first_map_.clear();
        first_map_ = first_map.clone();
        for symbol in symbols.iter() {
            match symbol {
                Symbol::TERMINAL(terminal) => {
                    first_map.insert(symbol.clone(), HashSet::from([terminal.clone()]));
                }
                Symbol::NONTERMINAL(_) => {
                    let p = productions_hashmap.get(symbol).unwrap();
                    p.iter().for_each(|body| {
                        if !body.is_empty() {
                            let first = body.first().unwrap();
                            let set_ = match first {
                                Symbol::TERMINAL(_) => HashSet::from([first.to_string()]),
                                Symbol::NONTERMINAL(_) => {
                                    let a: HashSet<String> =
                                        first_map.get(first).unwrap().iter().cloned().collect();
                                    a
                                }
                                _ => HashSet::new(),
                            };
                            let set = first_map.get_mut(symbol).unwrap();
                            set.extend(set_);
                        }
                    });
                }
                Symbol::NONE => continue,
            }
        }
    }
    println!("symbols --> {:#?}\nfirstmap --> {:#?}", symbols, first_map);
    first_map
}
