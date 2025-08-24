use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{
    first::compute_first_set,
    follow::compute_follow_set,
    item::{Item, ItemVecExtension},
    production::Production,
    state::{State, StateVecExtension},
    symbol::{unique_symbols, Symbol},
    traits::VecExtension,
};

#[derive(Debug)]
pub struct LR1_Parser<'a, AST, Token, TranslatorStack> {
    pub productions: &'a Vec<Production<AST, Token, TranslatorStack>>,
    pub LR1_automata: Vec<State<'a, AST, Token, TranslatorStack>>,
    pub symbols: HashSet<Symbol>, //every gramar symbol that exists in grammar
    pub follow_set: HashMap<Symbol, HashSet<String>>,
    pub first_set: HashMap<Symbol, HashSet<String>>,
    pub conflicts: bool,
}

impl<'a, AST, Token, TranslatorStack> LR1_Parser<'a, AST, Token, TranslatorStack>
where
    AST: Clone + Debug + PartialEq,
    Token: ToString + Debug + Clone + PartialEq,
    TranslatorStack: Clone + Debug + PartialEq,
{
    pub fn new(
        productions: &Vec<Production<AST, Token, TranslatorStack>>,
    ) -> LR1_Parser<AST, Token, TranslatorStack> {
        //collect all grammar symbols without duplicates
        let symbols: HashSet<Symbol> = unique_symbols(&productions);

        let first_set = compute_first_set(&productions);
        let follow_set = compute_follow_set(&productions);

        LR1_Parser {
            productions: productions,
            LR1_automata: Vec::new(),
            symbols,
            first_set,
            follow_set,
            conflicts: false,
        }
    }

    // Algorithm
    // void 𝐶𝐿𝑂𝑆𝑈𝑅𝐸(𝐼:items) {
    //   repeat
    //       for (each item [𝐴 → 𝛼.𝐵𝛽,𝑎] in 𝐼 )
    //           for ( each production [𝐵 → 𝛾] in 𝐺' )
    //               for ( each terminal 𝑏 in FIRST(𝛽𝑎) )
    //                   add [𝐵 → .𝛾,𝑏] to set 𝐼
    //   until no more items are added to 𝐼
    // }
    fn clousure(&self, items: &mut Vec<Item<'a, AST, Token, TranslatorStack>>) {
        let mut items_count = 0;
        let mut items_iterated_count = 0;
        while items.len().ne(&items_count) {
            items_count = items.len();
            let mut new_items = Vec::new();
            for items_index in items_iterated_count..items.len() {
                let item = items.get(items_index).unwrap();
                let B = item.next_symbol();
                let beta = item.production.body.get((item.cursor + 1) as usize);
                let first_of = if let Some(beta) = beta {
                    vec![beta.clone()]
                } else {
                    item.lookaheads.clone()
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
                    let mut lookaheads = Vec::new();
                    for first in first_of.iter() {
                        for terminal_b in self.first_set.get(first).unwrap().iter() {
                            lookaheads.push(Symbol::TERMINAL(terminal_b.to_string()));
                        }
                    }
                    let p = *b_production;
                    let item_ = Item {
                        production: p,
                        cursor: 0,
                        lookaheads: lookaheads,
                    };
                    if items.custom_contains(&item_) || new_items.custom_contains(&item_) {
                        continue;
                    }
                    new_items.push(item_);
                }
                items_iterated_count += 1;
            }
            items.extend(new_items);
        }
        items.merge_cores();
    }

    // Algorithm
    // State 𝐺𝑂𝑇𝑂(𝐼:items, 𝑋:symbol) {
    //   initialize 𝐽 to be the empty set;
    //   for ( each item [𝐴 → 𝛼.𝑋𝛽,𝑎] in 𝐼)
    //       add item [𝐴 → 𝛼𝑋.𝛽,𝑎] to set 𝐽;
    //   return 𝐶𝐿𝑂𝑆𝑈𝑅𝐸(𝐽);
    // }
    fn goto(
        &self,
        items: &Vec<Item<'a, AST, Token, TranslatorStack>>,
        symbol: &Symbol,
    ) -> State<'a, AST, Token, TranslatorStack> {
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
        let transition_productions = new_items.clone();
        let mut state = State::new(0, new_items.clone(), transition_productions);
        self.clousure(&mut new_items);
        state.items = new_items;
        state
    }

    // Algorithm
    // void 𝐼𝑇𝐸𝑀𝑆(𝐺') {
    //   initialize 𝐶 to { 𝐶𝐿𝑂𝑆𝑈𝑅𝐸({[𝑆' → .𝑆,$]}) };
    //   repeat
    //       for ( each set of items 𝐼 in 𝐶 )
    //           for ( each grammar symbol 𝑋 )
    //               if ( 𝐺𝑂𝑇𝑂(𝐼, 𝑋) is not empty and not in 𝐶 )
    //                   add 𝐺𝑂𝑇𝑂(𝐼, 𝑋) to 𝐶;
    //   until no new sets of items are added to 𝐶;
    // }
    fn items(&mut self) {
        let augmented_item: Item<AST, Token, TranslatorStack> = Item {
            production: self.productions.first().unwrap(),
            cursor: 0,
            lookaheads: vec![Symbol::TERMINAL(String::from("EOF"))],
        };
        let mut S0_items = vec![augmented_item];
        self.clousure(&mut S0_items);
        let mut LR1_automata = vec![State {
            transition_productions: vec![],
            index: 0,
            items: S0_items,
        }];
        let mut states_count = 0;
        let mut states_iterated_count = 0;
        while LR1_automata.len().ne(&states_count) {
            states_count = LR1_automata.len();
            let mut new_state = vec![];
            for states_index in states_iterated_count..LR1_automata.len() {
                let state = LR1_automata.get(states_index).unwrap();
                for symbol in self.symbols.iter() {
                    let goto_productions_state = self.goto(&state.items, symbol);
                    if !goto_productions_state.items.is_empty()
                        && !LR1_automata.custom_contains(&goto_productions_state)
                    {
                        new_state.push(goto_productions_state);
                    }
                }
                states_iterated_count += 1;
            }
            LR1_automata.extend(new_state);
        }
        LR1_automata
            .iter_mut()
            .enumerate()
            .for_each(|(index, state)| state.index = index);
        self.LR1_automata = LR1_automata;
    }

    // Algorithm
    // 𝐈𝐍𝐏𝐔𝐓 : An augmented grammar 𝐺'
    // 𝐎𝐔𝐓𝐏𝐔𝐓 : The 𝐿𝐴𝐿𝑅 parsing-table functions 𝐴𝐶𝑇𝐼𝑂𝑁 and 𝐺𝑂𝑇𝑂 for 𝐺'
    // 𝐌𝐄𝐓𝐇𝐎𝐃 :
    //  1. Construct 𝐶 = {𝐼₀,𝐼₁,...,𝐼ₙ}, the collection of sets of 𝐿𝑅(1) items
    //  2. For each core present among the set of 𝐿𝑅(1) items, find all sets
    //     having that core, and replace these sets by their union.
    //  3. Let 𝐶' = {𝐽₀,𝐽₁,...,𝐽ₙ} be the resulting sets of 𝐿𝑅(1) items.
    //  4. State 𝑖 of the parser is constructed from 𝐽ᵢ. The parsing action for
    //     state 𝑖 us determined as follows
    //     (a) If [𝐴 → 𝛼.𝑎𝛽,𝑏] is in 𝐽ᵢ and 𝐺𝑂𝑇𝑂(𝐽ᵢ,𝑎) = 𝐽ₖ, then set 𝐴𝐶𝑇𝐼𝑂𝑁[𝑖,𝑎]
    //         to "shift 𝑘". Here 𝑎 must be a terminal.
    //     (b) If [𝐴 → 𝛼.,𝑎] is in 𝐽ᵢ, 𝐴 ≠ 𝑆', then set 𝐴𝐶𝑇𝐼𝑂𝑁[𝑖,𝑎] to
    //         "reduce 𝐴 → 𝛼".
    //     (c) If [𝑆' → 𝑆.,$] is in 𝐽ᵢ, the set then set 𝐴𝐶𝑇𝐼𝑂𝑁[𝑖,𝑎] to "accept".
    //     If any conflicting actions result from above rules, we say the
    //     grammar is not 𝐿𝐴𝐿𝑅(1). The algorithm fails to produce a parser in this
    //     case.
    //  5. The goto transitions for state 𝑖 are constructed for all nonterminals
    //     𝐴 using the rule: If 𝐺𝑂𝑇𝑂(𝐽ᵢ,𝐴) = 𝐽ₖ, then 𝐺𝑂𝑇𝑂[𝑖,𝐴] = 𝑘.
    //  6. All entries not defined by rules (4) and (5) are made "error".
    //  7. Then intitial state of the parser is the one constructed from the set
    //     of items containing [𝑆' → .𝑆,$]
    pub fn construct_LALR_Table(&mut self) {
        self.items();
        self.LR1_automata.merge_sets();
        println!("LR_automata: {:#?}", self.LR1_automata);
        println!("LR_automata: {:#?}", self.LR1_automata.len());
    }
}
