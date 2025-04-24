use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::{production::Production, symbol::unique_symbols, terminal::Terminal, Symbol};

pub fn compute_first_set<T: PartialEq + Clone + Eq + Debug + Hash + Terminal<T>>(
    productions: &Vec<Production<T>>,
) -> HashMap<Symbol<T>, HashSet<T>> {
    let symbols = unique_symbols(productions);

    let mut first_map: HashMap<Symbol<T>, HashSet<T>> = HashMap::new();
    for symbol in symbols.iter() {
        match symbol {
            Symbol::TERMINAL(terminal) => {
                first_map.insert(symbol.clone(), HashSet::from([terminal.clone()]));
            }
            Symbol::NONTERMINAL(non_terminal) => {
                let filter_by_head: Vec<T> = productions
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
