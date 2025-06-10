use core::panic;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::Matches;
use std::sync::Arc;

use crate::action::Action;
use crate::conflict::ConflictType;
use crate::error::ParseError;
use crate::first::compute_first_set;
use crate::follow::compute_follow_set;
use crate::production::Production;
use crate::state::State;
use crate::symbol::unique_symbols;
use crate::symbol::Symbol;
use crate::terminal;
use crate::terminal::Terminal;

#[derive(Debug)]
pub struct Parser<T> {
    pub productions: Vec<Production<T>>,
    pub lr0_automaton: Vec<State<T>>,
    pub symbols: Vec<Symbol>, //every gramar symbol that exists in grammar
    pub follow_set: HashMap<Symbol, HashSet<String>>,
    pub first_set: HashMap<Symbol, HashSet<String>>,
    pub conflicts: bool,
}

impl<T> Parser<T>
where
    T: Clone,
{
    pub fn new(productions: Vec<Production<T>>) -> Parser<T> {
        let mut productions_ = productions.clone();
        let dummy = vec![1, 2];

        //creating augmented production
        let start_symbol = productions_.first().unwrap();
        let augmented_production: Production<T> = Production {
            head: String::from("S'"),
            body: vec![Symbol::NONTERMINAL(start_symbol.head.clone())],
            cursor_pos: 0,
            index: 0,
            error_message: None,
            action: Some(Arc::new(|dummy| println!("msdian"))),
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

    pub fn closure(&self, productions: &mut Vec<Production<T>>) {
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
                if symbols.contains(&symbol) {
                    continue;
                } else {
                    symbols.push(symbol.clone());
                }

                let matched_head_productions: Vec<Production<T>> = self
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
    pub fn goto(&self, productions: &Vec<Production<T>>, symbol: &Symbol) -> State<T> {
        let mut goto_productions: Vec<Production<T>> = productions
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
        let transition_productions = goto_productions.clone();
        self.closure(&mut goto_productions);
        State {
            state: 0, //Dont forget to update this according to the index in canonical collection
            productions: goto_productions,
            transition_symbol: symbol.clone(),
            transition_productions: transition_productions,
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
            transition_productions: vec![augmented_grammar.clone()],
            action: HashMap::new(),
            goto: HashMap::new(),
            conflicts: HashMap::new(),
        };
        let mut canonical_collection: Vec<State<T>> = vec![initial_state];
        loop {
            //this clone is for because we cant update the vector which is already in use
            let mut canonical_clone = canonical_collection.clone();
            //this loop is uneccesary it can be optimised
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
                break;
            }
        }
        self.lr0_automaton = canonical_collection;
    }

    pub fn parse<TokenType: Terminal + Debug + Clone>(
        &self,
        input: Vec<TokenType>,
        errors: &mut Vec<ParseError<TokenType>>,
    ) {
        let mut stack: Vec<State<T>> = Vec::new();
        let mut input_iter = input.iter();
        let mut current_input = input_iter.next().unwrap();
        let mut previous_input = current_input;
        let mut current_input_string = current_input.to_string_c();
        let mut top_state = self.lr0_automaton.first().unwrap();

        let mut translator_stack: Vec<T> = Vec::new();

        stack.push(top_state.clone());
        loop {
            top_state = stack.last().unwrap();

            if top_state.action.contains_key(&current_input_string) {
                match top_state.action.get(&current_input_string).unwrap() {
                    Action::SHIFT(state) => {
                        stack.push(self.lr0_automaton.get(state.clone()).unwrap().clone());
                        previous_input = current_input;
                        current_input = input_iter.next().unwrap();
                        current_input_string = current_input.to_string_c();
                    }
                    Action::REDUCE(production) => {
                        let production_ = self.productions.get(production.clone()).unwrap();
                        match &production_.action {
                            Some(actionaa) => (actionaa.as_ref())(&translator_stack),
                            None => {}
                        };
                        let pop_len = production_.body.len();
                        for _ in 0..pop_len {
                            stack.pop();
                        }
                        let stack_top = stack.last().unwrap();
                        let goto_state = stack_top.goto.get(&production_.head).unwrap();
                        stack.push(self.lr0_automaton.get(goto_state.clone()).unwrap().clone());
                    }
                    Action::ACCEPT => {
                        break;
                    }
                    _ => {}
                }
            } else {
                let mut input_symbol_skip_count = 0;
                let error_token = current_input;
                //error recovery
                //implement second method in this paper https://ieeexplore.ieee.org/document/6643853

                let mut error_message = counstruct_syntax_error_message(top_state);

                let deduced_productions = top_state.transition_productions.clone();
                let mut deduced_production: Option<Production<T>> = None;
                loop {
                    stack.pop();
                    top_state = stack.last().unwrap();
                    let mut contains = false;
                    for production in deduced_productions.iter() {
                        if top_state.goto.contains_key(&production.head) {
                            contains = true;
                            deduced_production = Some(production.clone());
                            break;
                        }
                    }
                    if contains {
                        break;
                    }
                }
                //skip input till input character contains in followset of ...
                //top_state transition symbol
                let error_production_follow_set = self
                    .follow_set
                    .get(&Symbol::NONTERMINAL(
                        deduced_production.clone().unwrap().head,
                    ))
                    .unwrap();
                loop {
                    if error_production_follow_set.contains(&current_input_string) {
                        //println!("error message:{:#?}", deduced_production);
                        if deduced_production.clone().unwrap().error_message.is_some() {
                            error_message =
                                deduced_production.unwrap().error_message.unwrap().clone();
                        }
                        if input_symbol_skip_count == 0 {
                            errors.push(ParseError {
                                token: previous_input.clone(),
                                message: error_message,
                                productionEnd: true,
                            });
                        } else {
                            errors.push(ParseError {
                                token: error_token.clone(),
                                message: error_message,
                                productionEnd: false,
                            });
                        }
                        break;
                    } else {
                        input_symbol_skip_count += 1;
                        previous_input = current_input;
                        current_input = input_iter.next().unwrap();
                        current_input_string = current_input.to_string_c();
                    }
                }
            }
        }
    }
}

fn counstruct_syntax_error_message<T>(state: &State<T>) -> String {
    let action_keys: Vec<String> = state.action.keys().cloned().collect();
    String::from("Expected ") + join_either_or(action_keys).as_str()
}

fn join_either_or(items: Vec<String>) -> String {
    match items.len() {
        0 => "".to_string(),
        1 => items[0].clone(),
        2 => format!("{} or {}", items[0], items[1]),
        _ => {
            let all_but_last = &items[..items.len() - 1];
            let last = &items[items.len() - 1];
            format!("{} or {}", all_but_last.join(", "), last)
        }
    }
}

fn is_items_in_canonical_collection<T: Clone>(
    states: Vec<State<T>>,
    item: &Vec<Production<T>>,
) -> (bool, Option<State<T>>) {
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

fn compute_shift_reduce_productions<T: Clone>(
    productions: &Vec<Production<T>>,
) -> (Vec<Production<T>>, Vec<Production<T>>) {
    let filter_by_dot_at_the_rightend =
        |production: &Production<T>| production.cursor_pos == production.body.len();

    let reduce_productions: Vec<Production<T>> = productions
        .iter()
        .cloned()
        .filter(filter_by_dot_at_the_rightend)
        .collect();

    let shift_productions: Vec<Production<T>> = productions
        .iter()
        .cloned()
        .filter(|p| !filter_by_dot_at_the_rightend(p))
        .collect();

    (reduce_productions, shift_productions)
}
