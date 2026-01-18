use indexmap::{IndexMap, IndexSet};

use crate::{
    grammar::Grammar,
    symbol::{SymbolId, EOF_SYMBOL_ID},
    token::TokenKind,
};

pub fn compute_first_set<AST, Token: TokenKind, TranslatorStack>(
    grammar: &Grammar<AST, Token, TranslatorStack>,
) -> IndexMap<SymbolId, IndexSet<SymbolId>> {
    let mut first_map: IndexMap<SymbolId, IndexSet<SymbolId>> = IndexMap::new();
    let mut first_map_: IndexMap<SymbolId, IndexSet<SymbolId>> = IndexMap::new();
    let mut productions_hashmap: IndexMap<SymbolId, Vec<Vec<SymbolId>>> = IndexMap::new();
    let mut symbols = vec![EOF_SYMBOL_ID];
    symbols.extend(grammar.symbols.terminals.clone());
    symbols.extend(grammar.symbols.non_terminals.clone());

    symbols.iter().for_each(|symbol| {
        first_map.insert(symbol.clone(), IndexSet::new());
        if grammar.symbols.non_terminal(symbol) {
            productions_hashmap.insert(symbol.clone(), Vec::new());
        }
    });

    for production in grammar.productions.vec.iter() {
        let p = productions_hashmap.get_mut(&production.head);
        if let Some(body) = p {
            body.push(production.body.clone());
        }
    }

    while first_map_.ne(&first_map) {
        first_map_.clear();
        first_map_ = first_map.clone();
        for symbol in symbols.iter() {
            match symbol {
                x if grammar.symbols.terminal(x) => {
                    first_map.insert(symbol.clone(), IndexSet::from([symbol.clone()]));
                }
                x if grammar.symbols.non_terminal(x) => {
                    let p = productions_hashmap.get(symbol).unwrap();
                    p.iter().for_each(|body| {
                        if !body.is_empty() {
                            let first = body.first().unwrap();
                            let set_ = match first {
                                x if grammar.symbols.terminal(x) => IndexSet::from([*x]),
                                x if grammar.symbols.non_terminal(x) => {
                                    first_map.get(first).unwrap().iter().cloned().collect()
                                }
                                _ => return,
                            };
                            let set = first_map.get_mut(symbol).unwrap();
                            set.extend(set_);
                        }
                    });
                }
                _ => {}
            }
        }
    }
    first_map
}
