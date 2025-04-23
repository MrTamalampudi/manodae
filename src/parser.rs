use std::collections;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr::hash;

use crate::first::compute_first_set;
use crate::follow;
use crate::follow::compute_follow_set;
use crate::production::Production;
use crate::symbol::unique_symbols;
use crate::Action;
use crate::State;
use crate::Symbol;
use crate::TokenType;

#[derive(Debug)]
pub struct Parser {
    pub productions: Vec<Production>,
    pub lr0_automaton: Vec<State>,
    pub symbols: Vec<Symbol>, //every gramar symbol that exists in grammar
    pub follow_set: HashMap<Symbol, HashSet<TokenType>>,
    pub first_set: HashMap<Symbol, HashSet<TokenType>>,
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
        };
        let mut canonical_collection: Vec<State> = vec![initial_state];
        loop {
            //this clone is for because we cant update the vector which is already in use
            let mut canonical_clone = canonical_collection.clone();
            for state in canonical_clone.iter_mut() {
                let (reduce_productions, shift_productions) =
                    compute_shift_reduce_productions(&state.productions);
                for symbol in self.symbols.iter() {
                    if symbol.is_terminal() {
                        for production in reduce_productions.iter() {
                            let state_ = canonical_collection.get_mut(state.state).expect("");
                            if production.head == String::from("S'")
                                && symbol.eq(&Symbol::TERMINAL(TokenType::EOF))
                            {
                                state_.action.insert(TokenType::EOF.clone(), Action::ACCEPT);
                            }
                            if production.head == String::from("S'") {
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
                                match symbol {
                                    Symbol::TERMINAL(terminal) => {
                                        state_.action.insert(
                                            terminal.clone(),
                                            Action::REDUCE(production.index),
                                        );
                                    }
                                    _ => {}
                                };
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
                        let state_ = canonical_collection.get_mut(state.state).expect("");
                        match symbol {
                            Symbol::TERMINAL(terminal) => {
                                state_
                                    .action
                                    .insert(terminal.clone(), Action::SHIFT(goto_state.state));
                            }
                            Symbol::NONTERMINAL(non_terminal) => {
                                state_.goto.insert(non_terminal.clone(), goto_state.state);
                            }
                            Symbol::NONE => {}
                        }
                    }
                    if !goto.productions.is_empty() && !is_item_in_collection {
                        goto.state = canonical_collection.len();
                        canonical_collection.push(goto.clone());
                        let state_ = canonical_collection.get_mut(state.state).expect("");
                        match symbol {
                            Symbol::TERMINAL(terminal) => {
                                state_
                                    .action
                                    .insert(terminal.clone(), Action::SHIFT(goto.state));
                            }
                            Symbol::NONTERMINAL(non_terminal) => {
                                state_.goto.insert(non_terminal.clone(), goto.state);
                            }
                            Symbol::NONE => {}
                        }
                    }
                }
            }
            if canonical_clone.len() == canonical_collection.len() {
                break;
            }
        }
        println!("{:#?}", canonical_collection);
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
