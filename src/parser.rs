use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{
    first::compute_first_set,
    follow::compute_follow_set,
    item::Item,
    production::Production,
    symbol::{unique_symbols, Symbol},
};

#[derive(Debug)]
pub struct LR1_Parser<'a, AST, Token, TranslatorStack> {
    pub productions: &'a Vec<Production<AST, Token, TranslatorStack>>,
    pub items: Items<'a, AST, Token, TranslatorStack>,
    pub LR1_automata: Vec<Items<'a, AST, Token, TranslatorStack>>,
    pub symbols: HashSet<Symbol>, //every gramar symbol that exists in grammar
    pub follow_set: HashMap<Symbol, HashSet<String>>,
    pub first_set: HashMap<Symbol, HashSet<String>>,
    pub conflicts: bool,
}

pub type Items<'a, AST, Token, TranslatorStack> = Vec<Item<'a, AST, Token, TranslatorStack>>;

impl<'a, AST, Token, TranslatorStack> LR1_Parser<'a, AST, Token, TranslatorStack>
where
    AST: Clone + Debug + PartialEq,
    Token: ToString + Debug + Clone + PartialEq,
    TranslatorStack: Clone + Debug + PartialEq,
{
    pub fn new(
        productions: &Vec<Production<AST, Token, TranslatorStack>>,
    ) -> LR1_Parser<AST, Token, TranslatorStack> {
        // let mut productions_ = eliminate_unit_productions(productions);
        // productions_ = eliminate_useless_productions(productions_);

        //collect all grammar symbols without duplicates
        let symbols: HashSet<Symbol> = unique_symbols(&productions);

        let first_set = compute_first_set(&productions);
        let follow_set = compute_follow_set(&productions);

        LR1_Parser {
            productions: productions,
            items: Vec::new(),
            LR1_automata: Vec::new(),
            symbols,
            first_set,
            follow_set,
            conflicts: false,
        }
    }

    //Algorithm
    //void CLOSURE(𝐼:items) {
    //  repeat
    //      for (each item [A → 𝛼.𝐵𝛽,𝑎] in 𝐼 )
    //          for ( each production [𝐵 → 𝛾] in 𝐺' )
    //              for ( each terminal 𝑏 in FIRST(𝛽𝑎) )
    //                  add [𝐵 → .𝛾,𝑏] to set 𝐼
    //  until no more items are added to 𝐼
    //}
    fn clousure(&self, items: &mut Items<'a, AST, Token, TranslatorStack>) {
        let mut items_count = 0;
        let mut items_iterated_count = 0;
        while items.len().ne(&items_count) {
            items_count = items.len();
            let mut new_items = vec![];
            for items_index in items_iterated_count..items.len() {
                let item = items.get(items_index).unwrap();
                let B = item.next_symbol();
                let beta = item.production.body.get((item.cursor + 1) as usize);
                let first_of = if let Some(beta) = beta {
                    beta
                } else {
                    item.lookaheads.first().unwrap()
                };
                if let None = B {
                    continue;
                }
                let production_B = B.unwrap();
                if production_B.is_terminal() {
                    continue;
                }
                let b_productions: Vec<_> = self
                    .productions
                    .iter()
                    .filter(|production| production.head.eq(&production_B.to_string()))
                    .collect();
                for b_production in b_productions.iter() {
                    for terminal_b in self.first_set.get(first_of).unwrap().iter() {
                        let p = *b_production;
                        let item_ = Item {
                            production: p,
                            cursor: 0,
                            lookaheads: vec![Symbol::TERMINAL(terminal_b.to_string())],
                        };
                        if !items.contains(&item_) {
                            new_items.push(item_);
                        }
                    }
                }
                items_iterated_count += 1;
            }
            items.extend(new_items);
        }
    }

    //Algorithm
    //State GOTO(𝐼:items, 𝑋:symbol) {
    //  initialize 𝐽 to be the empty set;
    //  for ( each item [A → 𝛼.𝑋𝛽,𝑎] in 𝐼)
    //      add item [A → 𝛼𝑋.𝛽,𝑎] to set 𝐽;
    //  return CLOSURE(𝐽);
    //}
    fn goto(
        &self,
        items: &Items<'a, AST, Token, TranslatorStack>,
        symbol: &Symbol,
    ) -> Vec<Item<'a, AST, Token, TranslatorStack>> {
        let mut new_items = vec![];
        for mut item in items.iter().cloned() {
            let item_symbol = item.next_symbol();
            if item_symbol.is_none() {
                continue;
            }
            if symbol != item_symbol.unwrap() {
                continue;
            }
            item.advance_cursor();
            new_items.push(item);
        }
        self.clousure(&mut new_items);
        new_items
    }

    //Algorithm
    //void items(𝐺') {
    //  initialize 𝐶 to { CLOSURE({[𝑆' → .𝑆,$]}) };
    //  repeat
    //      for ( each set of items 𝐼 in 𝐶 )
    //          for ( each grammar symbol 𝑋 )
    //              if ( GOTO(𝐼, 𝑋) is not empty and not in 𝐶 )
    //                  add GOTO(𝐼, 𝑋) to 𝐶;
    //  until no new sets of items are added to 𝐶;
    //}
    pub fn items(&mut self) {
        let augmented_item: Item<AST, Token, TranslatorStack> = Item {
            production: self.productions.first().unwrap(),
            cursor: 0,
            lookaheads: vec![Symbol::TERMINAL(String::from("EOF"))],
        };
        let mut S0_items = vec![augmented_item];
        self.clousure(&mut S0_items);
        let mut LR1_automata = vec![S0_items];
        let mut states_count = 0;
        let mut states_iterated_count = 0;
        while LR1_automata.len().ne(&states_count) {
            states_count = LR1_automata.len();
            let mut new_items = vec![];
            for states_index in states_iterated_count..LR1_automata.len() {
                let items = LR1_automata.get(states_index).unwrap();
                for symbol in self.symbols.iter() {
                    let goto_productions = self.goto(items, symbol);
                    if !goto_productions.is_empty() && !LR1_automata.contains(&goto_productions) {
                        new_items.push(goto_productions);
                    }
                }
                states_iterated_count += 1;
            }
            LR1_automata.extend(new_items);
        }
        self.LR1_automata = LR1_automata;
    }
}
