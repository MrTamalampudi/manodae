use std::collections::{HashMap, HashSet};

use crate::{
    first::compute_first_set,
    production_LR1::Production_LR1,
    symbol::{unique_symbols, Symbol},
};

pub fn compute_follow_set<AST, Token, TranslatorStack>(
    productions: &Vec<&Production_LR1<AST, Token, TranslatorStack>>,
) -> HashMap<Symbol, HashSet<String>> {
    let symbols = unique_symbols(productions);
    let non_terminals: Vec<Symbol> = symbols
        .iter()
        .cloned()
        .filter_map(|symbol| match symbol {
            Symbol::NONTERMINAL(nt) => Some(Symbol::NONTERMINAL(nt)),
            _ => None,
        })
        .collect();

    let mut follow_map: HashMap<Symbol, HashSet<String>> = HashMap::new();

    //populate with empty vectors
    non_terminals.iter().for_each(|symbol| {
        follow_map.insert(symbol.clone(), HashSet::new());
    });

    let augment_production: Option<&&Production_LR1<AST, Token, TranslatorStack>> = productions
        .iter()
        .filter(|prod| prod.head.eq(&String::from("S'")))
        .next();

    match augment_production {
        Some(prod) => {
            let start = prod.body.first().unwrap();
            match start {
                //eofff
                Symbol::NONTERMINAL(_) => {
                    follow_map.insert(start.clone(), HashSet::from([String::from("EOF")]));
                }
                _ => {}
            };
        }
        None => {}
    };

    let first = compute_first_set(productions);

    //A -> a B D , then everything in First(D) is in Follow(B)
    //this loop implements above def
    for nt in non_terminals.iter() {
        for production in productions.iter() {
            let with_indexes: Vec<(usize, &Symbol)> = production
                .body
                .iter()
                .enumerate()
                .filter(|(_, symbol)| symbol.eq(&nt))
                .map(|(index, symbol)| (index, symbol))
                .collect();

            //not so good logic but hope it works
            for (index, _) in with_indexes.iter() {
                if index.clone() == production.body.len() - 1 {
                    continue;
                } else {
                    let next_ = production.body.get(index + 1).unwrap();
                    let first_ = first.get(next_).unwrap();
                    follow_map
                        .entry(nt.clone())
                        .and_modify(|token_types| token_types.extend(first_.clone()))
                        .or_insert(HashSet::from_iter(first_.iter().cloned()));
                }
            }
        }
    }

    //A->aB then everything in  Follow(A) is in Follow(B)
    loop {
        let follow_count_func = |follow_map: &HashMap<Symbol, HashSet<String>>| {
            follow_map
                .values()
                .flat_map(|token_types| token_types)
                .count()
        };
        let follow_map_count_before = follow_count_func(&follow_map);

        for production in productions.iter() {
            if production.head.eq(&String::from("S'")) {
                continue;
            } else {
                let last_symbol = production.body.last().unwrap();
                let follow_head =
                    match follow_map.get(&Symbol::NONTERMINAL(production.head.clone())) {
                        Some(set) => set.clone(),
                        None => continue,
                    };
                follow_map
                    .entry(last_symbol.clone())
                    .and_modify(|token_types| token_types.extend(follow_head));
            }
        }
        if follow_map_count_before == follow_count_func(&follow_map) {
            break;
        }
    }

    follow_map
}
