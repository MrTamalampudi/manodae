use std::{ops::Deref, rc::Rc};

use indexmap::{IndexMap, IndexSet};

use crate::{first::compute_first_set, grammar::Grammar, production::Production, symbol::Symbol};

pub fn compute_follow_set<AST, Token, TranslatorStack>(
    grammar: &Grammar<AST, Token, TranslatorStack>,
) -> IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> {
    let mut follow_map: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>> = IndexMap::new();

    //populate with empty vectors
    grammar.non_terminals.iter().for_each(|symbol| {
        follow_map.insert(Rc::clone(symbol), IndexSet::new());
    });

    let augment_production: Option<&Rc<Production<AST, Token, TranslatorStack>>> = grammar
        .productions
        .iter()
        .filter(|prod| prod.head.eq(&String::from("S'")))
        .next();

    match augment_production {
        Some(prod) => {
            let start = prod.body.first().unwrap();
            match start.deref() {
                //eofff
                Symbol::NONTERMINAL(_) => {
                    follow_map.insert(
                        start.clone(),
                        IndexSet::from([Rc::new(Symbol::TERMINAL(String::from("EOF")))]),
                    );
                }
                _ => {}
            };
        }
        None => {}
    };

    let first = compute_first_set(grammar);

    //A -> a B D , then everything in First(D) is in Follow(B)
    //this loop implements above def
    for non_terminal in grammar.non_terminals.iter() {
        for production in grammar.productions.iter() {
            let with_indexes: Vec<(usize, &Rc<Symbol>)> = production
                .body
                .iter()
                .enumerate()
                .filter(|(_, symbol)| symbol.eq(&non_terminal))
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
                        .entry(non_terminal.clone())
                        .and_modify(|token_types| token_types.extend(first_.clone()))
                        .or_insert(IndexSet::from_iter(first_.iter().cloned()));
                }
            }
        }
    }

    //A->aB then everything in  Follow(A) is in Follow(B)
    loop {
        let follow_count_func = |follow_map: &IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>>| {
            follow_map
                .values()
                .flat_map(|token_types| token_types)
                .count()
        };
        let follow_map_count_before = follow_count_func(&follow_map);

        for production in grammar.productions.iter() {
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
