use std::collections::{HashMap, HashSet};

use crate::{production::Production, symbol::unique_symbols, Symbol};

pub fn compute_first_set(productions: &Vec<Production>) -> HashMap<Symbol, HashSet<String>> {
    let symbols = unique_symbols(productions);

    let mut first_map: HashMap<Symbol, HashSet<String>> = HashMap::new();
    for symbol in symbols.iter() {
        match symbol {
            Symbol::TERMINAL(terminal) => {
                first_map.insert(symbol.clone(), HashSet::from([terminal.clone()]));
            }
            Symbol::NONTERMINAL(non_terminal) => {
                let filter_by_head: Vec<String> = productions
                    .iter()
                    .filter(|prod| prod.head.eq(non_terminal))
                    .filter_map(|prod| match prod.body.first().unwrap() {
                        Symbol::TERMINAL(terminal) => Some(terminal.clone()),
                        _ => None,
                    })
                    .collect();

                first_map.insert(
                    symbol.clone(),
                    HashSet::from_iter(filter_by_head.iter().cloned()),
                );
            }
            Symbol::NONE => continue,
        }
    }
    first_map
}
