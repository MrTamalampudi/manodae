use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::{
    first::compute_first_set,
    grammar::Grammar,
    production::Production,
    symbol::{SymbolId, AUGMENT_START_SYMBOL_ID, EOF_SYMBOL_ID},
};

pub fn compute_follow_set<AST, Token, TranslatorStack>(
    grammar: &Grammar<AST, Token, TranslatorStack>,
) -> IndexMap<SymbolId, IndexSet<SymbolId>> {
    let mut follow_map: IndexMap<SymbolId, IndexSet<SymbolId>> = IndexMap::new();

    //populate with empty vectors
    grammar.symbols.non_terminals.iter().for_each(|symbol| {
        follow_map.insert(*symbol, IndexSet::new());
    });

    let augment_production: Option<&Rc<Production<AST, Token, TranslatorStack>>> = grammar
        .productions
        .iter()
        .filter(|prod| prod.head.eq(&AUGMENT_START_SYMBOL_ID))
        .next();

    match augment_production {
        Some(prod) => {
            let start = prod.body.first().unwrap();
            match start {
                //eofff
                x if grammar.symbols.non_terminal(x) => {
                    follow_map.insert(start.clone(), IndexSet::from([EOF_SYMBOL_ID]));
                }
                _ => {}
            };
        }
        None => {}
    };

    let first = compute_first_set(grammar);

    //A -> a B D , then everything in First(D) is in Follow(B)
    //this loop implements above def
    for non_terminal in grammar.symbols.non_terminals.iter() {
        for production in grammar.productions.iter() {
            let with_indexes: Vec<(usize, SymbolId)> = production
                .body
                .iter()
                .enumerate()
                .filter(|(_, symbol)| symbol.eq(&non_terminal))
                .map(|(index, symbol)| (index, *symbol))
                .collect();

            //not so good logic but hope it works
            for (index, _) in with_indexes.iter() {
                if index.clone() == production.body.len() - 1 {
                    continue;
                } else {
                    let next_ = production.body.get(index + 1).unwrap();
                    let first_ = first.get(next_).unwrap();
                    follow_map
                        .entry(non_terminal.clone())
                        .and_modify(|token_types| token_types.extend(first_.clone()))
                        .or_insert(IndexSet::from_iter(first_.iter().cloned()));
                }
            }
        }
    }

    //A->aB then everything in  Follow(A) is in Follow(B)
    loop {
        let follow_count_func = |follow_map: &IndexMap<SymbolId, IndexSet<SymbolId>>| {
            follow_map
                .values()
                .flat_map(|token_types| token_types)
                .count()
        };
        let follow_map_count_before = follow_count_func(&follow_map);

        for production in grammar.productions.iter() {
            if production.head.eq(&AUGMENT_START_SYMBOL_ID) {
                continue;
            } else {
                let last_symbol = production.body.last().unwrap();
                let follow_head = match follow_map.get(&production.head) {
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
