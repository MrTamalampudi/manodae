use std::collections::{HashMap, HashSet};

use crate::{
    production::Production,
    symbol::{unique_symbols, Symbol},
};

pub fn compute_first_set<T, TokenType>(
    productions: &Vec<Production<T, TokenType>>,
) -> HashMap<Symbol, HashSet<String>> {
    let symbols = unique_symbols(productions);

    let mut first_map: HashMap<Symbol, HashSet<String>> = HashMap::new();
    for symbol in symbols.iter() {
        match symbol {
            Symbol::TERMINAL(terminal) => {
                first_map.insert(symbol.clone(), HashSet::from([terminal.clone()]));
            }
            Symbol::NONTERMINAL(non_terminal) => {
                let s_set = first_recursive(productions, non_terminal);
                first_map.insert(symbol.clone(), HashSet::from_iter(s_set.iter().cloned()));
            }
            Symbol::NONE => continue,
        }
    }
    first_map
}

fn first_recursive<T, TokenType>(
    productions: &Vec<Production<T, TokenType>>,
    non_terminal: &String,
) -> HashSet<String> {
    let filter_by_non_terminal: Vec<&Production<T, TokenType>> = productions
        .iter()
        .filter(|p| p.head.eq(non_terminal))
        .collect();

    let mut first_set: HashSet<String> = HashSet::new();

    for production in filter_by_non_terminal.iter() {
        let first_symbol = production.body.first().unwrap();
        match first_symbol {
            Symbol::TERMINAL(terminal) => {
                first_set.insert(terminal.clone());
            }
            Symbol::NONTERMINAL(non_terminal) => {
                first_set.extend(first_recursive(productions, non_terminal));
            }
            Symbol::NONE => continue,
        };
    }

    first_set
}
