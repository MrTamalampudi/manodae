use core::panic;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::Matches;

use crate::action::Action;
use crate::conflict::ConflictType;
use crate::first::compute_first_set;
use crate::follow::compute_follow_set;
use crate::production::Production;
use crate::state::State;
use crate::symbol::unique_symbols;
use crate::terminal;
use crate::terminal::Terminal;
use crate::Symbol;
use crate::TokenType;

#[derive(Debug)]
pub struct Parser {
    pub productions: Vec<Production>,
    pub lr0_automaton: Vec<State>,
    pub symbols: Vec<Symbol>, //every gramar symbol that exists in grammar
    pub follow_set: HashMap<Symbol, HashSet<String>>,
    pub first_set: HashMap<Symbol, HashSet<String>>,
    pub conflicts: bool,
}

impl Parser {
    pub fn new(productions: Vec<Production>) -> Parser {
        let mut productions_ = productions.clone();

        //creating augmented production
        let start_symbol = productions_.first().unwrap();
        let augmented_production: Production = Production {
            head: String::from("S'"),
            body: vec![Symbol::NONTERMINAL(start_symbol.head.clone())],
            cursor_pos: 0,
            index: 0,
        };
        productions_.insert(0, augmented_production);

        //collect all grammar symbols without duplicates
        let symbols: Vec<Symbol> = unique_symbols(&productions_);

        let first_set = compute_first_set(&productions_);
        let follow_set = compute_follow_set(&productions_);

        Parser {
            productions: productions_,
            lr0_automaton: Vec::new(),
            symbols,
            first_set,
            follow_set,
            conflicts: false,
        }
    }

    pub fn closure(&self, productions: &mut Vec<Production>) {
        let mut symbols: Vec<String> = vec![];
        loop {
            let productions_ = productions.clone();
            for production in productions_.iter() {
                let symbol = match production.next_symbol() {
                    Some(symbol) => match symbol {
                        Symbol::NONTERMINAL(head) => head,
                        _ => continue,
                    },
                    None => continue,
                };

                //this is for not to add already added productions
                //removing this block causes infinite loop
                if symbols.contains(symbol) {
                    continue;
                } else {
                    symbols.push(symbol.clone());
                }

                let matched_head_productions: Vec<Production> = self
                    .productions
                    .iter()
                    .cloned()
                    .filter(|production| production.head.eq(symbol))
                    .collect();

                productions.extend(matched_head_productions);
            }

            if productions.len() == productions_.len() {
                break;
            }
        }
    }

    //GOTO(Item,Symbol) is defined to be the closure of the set of
    //all items [A -> aX.B] such that [A -> a.XB] is in I {ref:ullman dragon book}
    pub fn goto(&self, productions: &Vec<Production>, symbol: &Symbol) -> State {
        let mut goto_productions = productions
            .iter()
            .filter(|production| match production.next_symbol() {
                Some(symbol_) => symbol.eq(symbol_),
                None => false,
            })
            .map(|production| {
                let mut temp = production.clone();
                temp.advance_cursor();
                temp
            })
            .collect();
        self.closure(&mut goto_productions);
        State {
            state: 0, //Dont forget to update this according to the index in canonical collection
            productions: goto_productions,
            transition_symbol: symbol.clone(),
            action: HashMap::new(),
            goto: HashMap::new(),
            conflicts: HashMap::new(),
        }
    }

    pub fn compute_lr0_items(&mut self) {
        let augmented_grammar = self.productions.first().unwrap();
        let mut state1 = vec![augmented_grammar.clone()];
        self.closure(&mut state1);
        let initial_state = State {
            state: 0,
            productions: state1,
            transition_symbol: Symbol::NONE,
            action: HashMap::new(),
            goto: HashMap::new(),
            conflicts: HashMap::new(),
        };
        let mut canonical_collection: Vec<State> = vec![initial_state];
        let mut state_index = 0;
        loop {
            //this clone is for because we cant update the vector which is already in use
            let mut canonical_clone = canonical_collection.clone();
            for state_clone in canonical_clone.iter() {
                let (reduce_productions, shift_productions) =
                    compute_shift_reduce_productions(&state_clone.productions);
                for symbol in self.symbols.iter() {
                    if let Symbol::TERMINAL(terminal) = symbol {
                        for production in reduce_productions.iter() {
                            let state = canonical_collection.get_mut(state_clone.state).expect("");
                            //eofff
                            if production.is_augment_production()
                                && terminal.eq(&String::from("EOF"))
                            {
                                if let Some(action) = state.action.get("EOF") {
                                    if !matches!(action, Action::ACCEPT) {
                                        self.conflicts = true;
                                        state.conflicts.insert(
                                            String::from("EOF"),
                                            ConflictType::RR([action.clone(), Action::ACCEPT]),
                                        );
                                    }
                                } else {
                                    state.action.insert(String::from("EOF"), Action::ACCEPT);
                                }
                            }
                            if production.is_augment_production() {
                                continue;
                            }

                            let set = self
                                .follow_set
                                .get(&Symbol::NONTERMINAL(production.head.clone()))
                                .expect(format!("{}", production.head.clone()).as_str());

                            let follow_set_contains = match symbol {
                                Symbol::TERMINAL(terminal) => set.contains(terminal),
                                _ => false,
                            };

                            if follow_set_contains {
                                if let Symbol::TERMINAL(terminal) = symbol {
                                    match state.action.get(terminal) {
                                        Some(entry) => match entry {
                                            Action::REDUCE(index) if index != &production.index => {
                                                self.conflicts = true;
                                                state.conflicts.insert(
                                                    terminal.clone(),
                                                    ConflictType::SR([
                                                        entry.clone(),
                                                        Action::REDUCE(production.index),
                                                    ]),
                                                );
                                            }
                                            Action::REDUCE(_) => {}
                                            _ => {
                                                panic!("conflict")
                                            }
                                        },
                                        None => {
                                            state.action.insert(
                                                terminal.clone(),
                                                Action::REDUCE(production.index),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    //shift and goto
                    let mut goto = self.goto(&shift_productions, symbol);
                    let (is_item_in_collection, state__) = is_items_in_canonical_collection(
                        canonical_collection.clone(),
                        &goto.productions,
                    );
                    if !goto.productions.is_empty() && is_item_in_collection {
                        let goto_state = state__.unwrap();
                        let state = canonical_collection.get_mut(state_clone.state).expect("");
                        match symbol {
                            Symbol::TERMINAL(terminal) => {
                                if let Some(entry) = state.action.get(terminal) {
                                    match entry {
                                        Action::SHIFT(state_index)
                                            if state_index != &goto_state.state =>
                                        {
                                            panic!("conflict")
                                        }
                                        Action::SHIFT(_) => {}
                                        Action::REDUCE(_) => {
                                            self.conflicts = true;
                                            state.conflicts.insert(
                                                terminal.clone(),
                                                ConflictType::SR([
                                                    entry.clone(),
                                                    Action::SHIFT(goto_state.state),
                                                ]),
                                            );
                                        }
                                        _ => {
                                            panic!("conflict")
                                        }
                                    }
                                }
                                state
                                    .action
                                    .insert(terminal.clone(), Action::SHIFT(goto_state.state));
                            }
                            Symbol::NONTERMINAL(non_terminal) => {
                                state.goto.insert(non_terminal.clone(), goto_state.state);
                            }
                            Symbol::NONE => {}
                        }
                    }
                    if !goto.productions.is_empty() && !is_item_in_collection {
                        goto.state = canonical_collection.len();
                        canonical_collection.push(goto.clone());
                        let state = canonical_collection.get_mut(state_clone.state).expect("");
                        match symbol {
                            Symbol::TERMINAL(terminal) => {
                                if let Some(entry) = state.action.get(terminal) {
                                    match entry {
                                        Action::SHIFT(state_index)
                                            if state_index != &goto.state =>
                                        {
                                            self.conflicts = true;
                                        }
                                        Action::SHIFT(_) => {}
                                        Action::REDUCE(_) => {
                                            self.conflicts = true;
                                            state.conflicts.insert(
                                                terminal.clone(),
                                                ConflictType::SR([
                                                    entry.clone(),
                                                    Action::SHIFT(goto.state),
                                                ]),
                                            );
                                        }
                                        _ => {
                                            panic!("conflict")
                                        }
                                    }
                                }
                                state
                                    .action
                                    .insert(terminal.clone(), Action::SHIFT(goto.state));
                            }
                            Symbol::NONTERMINAL(non_terminal) => {
                                state.goto.insert(non_terminal.clone(), goto.state);
                            }
                            Symbol::NONE => {}
                        }
                    }
                }
            }
            if canonical_clone.len() == canonical_collection.len() {
                println!(
                    "clone len {} coll len {}",
                    canonical_clone.len(),
                    canonical_collection.len()
                );
                break;
            }
            state_index += 1;
        }
        self.lr0_automaton = canonical_collection;
    }

    pub fn parse<T: Terminal>(&self, input: Vec<T>) {
        let mut stack: Vec<State> = Vec::new();
        let mut input_iter = input.iter();
        let mut a = input_iter.next().unwrap().to_string();
        let mut top_state = self.lr0_automaton.first().unwrap();
        stack.push(top_state.clone());
        loop {
            top_state = stack.last().unwrap();
            if top_state.action.contains_key(&a) {
                match top_state.action.get(&a).unwrap() {
                    Action::SHIFT(state) => {
                        stack.push(self.lr0_automaton.get(state.clone()).unwrap().clone());
                        a = input_iter.next().unwrap().to_string();
                    }
                    Action::REDUCE(production) => {
                        let production_ = self.productions.get(production.clone()).unwrap();
                        let pop_len = production_.body.len();
                        for _ in 0..pop_len {
                            stack.pop();
                        }
                        let stack_top = stack.last().unwrap();
                        let goto_state = stack_top.goto.get(&production_.head).unwrap();
                        stack.push(self.lr0_automaton.get(goto_state.clone()).unwrap().clone());
                    }
                    Action::ACCEPT => {
                        println!("hell yeah");
                        break;
                    }
                    _ => {}
                }
            } else {
                println!("expected {:#?} actual {:#?}", top_state.action.keys(), a);
                break;
            }
        }
    }
}

fn is_items_in_canonical_collection(
    states: Vec<State>,
    item: &Vec<Production>,
) -> (bool, Option<State>) {
    let mut contains = false;
    for state in states.iter() {
        if state.productions.len() != item.len() {
            continue;
        }
        for production in item.iter() {
            if state.productions.contains(production) {
                contains = true;
            } else {
                contains = false;
                break;
            }
        }
        if contains {
            return (contains, Some(state.clone()));
        }
    }
    (contains, None)
}

fn compute_shift_reduce_productions(
    productions: &Vec<Production>,
) -> (Vec<Production>, Vec<Production>) {
    let filter_by_dot_at_the_rightend =
        |production: &Production| production.cursor_pos == production.body.len();

    let reduce_productions: Vec<Production> = productions
        .iter()
        .cloned()
        .filter(filter_by_dot_at_the_rightend)
        .collect();

    let shift_productions: Vec<Production> = productions
        .iter()
        .cloned()
        .filter(|p| !filter_by_dot_at_the_rightend(p))
        .collect();

    (reduce_productions, shift_productions)
}
