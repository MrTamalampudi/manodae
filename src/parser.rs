use std::{cell::RefCell, fmt::Debug, process::exit, rc::Rc};

use indexmap::{IndexMap, IndexSet};

use crate::{
    action::Action,
    error::ParseError,
    first::compute_first_set,
    follow::compute_follow_set,
    grammar::Grammar,
    interner::Interner,
    item::{Item, ItemVecExtension},
    production::{ProductionId, AUGMENT_PRODUCTION_ID},
    state::{State, StateId, StateVecExtension, States},
    symbol::{Symbol, SymbolId, AUGMENT_START_SYMBOL_ID, EOF_SYMBOL_ID, ERROR_SYMBOL_ID},
    token_kind::TokenKind,
};

#[derive(Debug, Clone)]
pub struct LR1_Parser<AST, Token, TranslatorStack> {
    pub grammar: Grammar<AST, Token, TranslatorStack>,
    pub LR1_automata: States,
    pub follow_set: IndexMap<SymbolId, IndexSet<SymbolId>>,
    pub first_set: IndexMap<SymbolId, IndexSet<SymbolId>>,
    pub conflicts: bool,
    pub goto: IndexMap<StateId, IndexMap<SymbolId, StateId>>,
    pub action: IndexMap<StateId, IndexMap<SymbolId, Action>>,
    //used only when constructing table, no need for parsing
    pub item_closure_map: IndexMap<Item, Vec<Item>>,
    //used only when constructing table, no need for parsing
    pub closure_map: IndexMap<Vec<Item>, Vec<Item>>,
}

impl<AST, Token, TranslatorStack> LR1_Parser<AST, Token, TranslatorStack>
where
    AST: Clone + Debug + PartialEq,
    Token: ToString + Debug + Clone + PartialEq + TokenKind,
    TranslatorStack: Clone + Debug + PartialEq,
{
    pub fn new(
        grammar: Grammar<AST, Token, TranslatorStack>,
    ) -> LR1_Parser<AST, Token, TranslatorStack> {
        //collect all grammar symbols without duplicates

        let first_set = compute_first_set(&grammar);
        let follow_set = compute_follow_set(&grammar);

        let mut production_head_map: IndexMap<SymbolId, IndexSet<ProductionId>> = IndexMap::new();

        grammar.productions.map.iter().for_each(|(production, id)| {
            production_head_map
                .entry(production.head)
                .and_modify(|entry| {
                    entry.insert(*id);
                })
                .or_insert(IndexSet::from([*id]));
        });

        let mut a = LR1_Parser {
            grammar,
            LR1_automata: States::new(),
            first_set,
            follow_set,
            conflicts: false,
            action: IndexMap::new(),
            goto: IndexMap::new(),
            item_closure_map: IndexMap::new(),
            closure_map: IndexMap::new(),
        };
        a.grammar.production_head_map = production_head_map;
        a.construct_LALR_Table();
        a
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
    fn clousure(&mut self, items: &mut Vec<Item>) {
        let mut items_count = 0;
        let mut items_iterated_count = 0;
        while items.len().ne(&items_count) {
            items_count = items.len();
            let mut new_items: Vec<Item> = Vec::new();
            for items_index in items_iterated_count..items.len() {
                let item = items.get(items_index).unwrap();
                let existing_item = self.item_closure_map.get(item);
                if let Some(ei) = existing_item {
                    new_items.extend(ei.clone());
                } else {
                    let B = item.next_symbol(&self.grammar.productions);
                    let production = self.grammar.productions.lookup(item.production);
                    let beta = production.body.get((item.cursor + 1) as usize);
                    let first_of = if let Some(beta) = beta {
                        vec![beta.clone()]
                    } else {
                        item.lookaheads.clone()
                    };
                    if let None = B {
                        continue;
                    }
                    let B = B.unwrap();
                    if self.grammar.symbols.terminal(&B) {
                        continue;
                    }
                    let b_productions: &IndexSet<_> =
                        self.grammar.production_head_map.get(&B).unwrap();
                    let mut lookaheads = Vec::new();
                    for first in first_of.iter() {
                        for terminal_b in self.first_set.get(first).unwrap().iter() {
                            lookaheads.push(*terminal_b);
                        }
                    }
                    let mut ni = vec![];
                    for b_production in b_productions.iter() {
                        let item_ = Item {
                            production: *b_production,
                            cursor: 0,
                            lookaheads: lookaheads.clone(),
                        };
                        if items.contains(&item_) || new_items.contains(&item_) {
                            continue;
                        }
                        ni.push(item_);
                    }
                    self.item_closure_map.insert(item.clone(), ni.clone());
                    new_items.extend(ni.clone());
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
        &mut self,
        state_: &Rc<RefCell<State>>,
        symbol: SymbolId,
    ) -> Option<Rc<RefCell<State>>> {
        let mut new_items = vec![];
        for item in state_.borrow().items.iter() {
            let item_symbol = item.next_symbol(&self.grammar.productions);
            if item_symbol.is_none() {
                continue;
            }
            if symbol != item_symbol.unwrap() {
                continue;
            }
            let mut item = item.clone();
            item.advance_cursor();
            new_items.push(item);
        }
        if new_items.is_empty() {
            return None;
        }
        let transition_productions = new_items.clone();
        let exisisting_closure_map = self.closure_map.get(&new_items);
        if let Some(ecm) = exisisting_closure_map {
            new_items = ecm.clone();
        } else {
            self.clousure(&mut new_items);
            self.closure_map
                .insert(transition_productions.clone(), new_items.clone());
        }
        let state = State::new(0, new_items, symbol);
        Some(Rc::new(RefCell::new(state)))
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
        let augmented_item: Item = Item {
            production: AUGMENT_PRODUCTION_ID,
            cursor: 0,
            lookaheads: vec![EOF_SYMBOL_ID],
        };
        let mut S0_items = vec![augmented_item];
        self.clousure(&mut S0_items);
        let mut LR1_automata = vec![Rc::new(RefCell::new(State {
            transition_symbol: AUGMENT_START_SYMBOL_ID, // Dummy symbol
            index: 0,
            items: S0_items,
            outgoing: IndexMap::new(),
            incoming: vec![],
        }))];
        let mut goto_set = IndexMap::new();
        let mut states_count = 0;
        let mut states_iterated_count = 0;
        let mut symbols = vec![];
        symbols.extend(self.grammar.symbols.non_terminals.clone());
        symbols.extend(self.grammar.symbols.terminals.clone());
        while LR1_automata.len().ne(&states_count) {
            states_count = LR1_automata.len();
            let mut new_state = vec![];
            for states_index in states_iterated_count..LR1_automata.len() {
                let state = LR1_automata.get(states_index).unwrap();
                let mut goto_map = IndexMap::new();
                for symbol in symbols.iter() {
                    let items = state.borrow().items.clone();
                    let es = goto_set.get(&(items.clone(), symbol));
                    if let Some(es) = es {
                        goto_map.insert(*symbol, Rc::clone(es));
                    } else {
                        let goto_productions_state = self.goto(&state, *symbol);
                        if goto_productions_state.is_none() {
                            continue;
                        }
                        let goto_productions_state = goto_productions_state.unwrap();
                        goto_set.insert((items, symbol), Rc::clone(&goto_productions_state));
                        goto_map.insert(*symbol, Rc::clone(&goto_productions_state));
                        new_state.push(Rc::clone(&goto_productions_state));
                    }
                }
                let state = LR1_automata.get(states_index).unwrap();
                state.borrow_mut().outgoing = goto_map;
                states_iterated_count += 1;
            }
            LR1_automata.extend(new_state);
        }
        LR1_automata
            .iter_mut()
            .enumerate()
            .for_each(|(index, state)| state.borrow_mut().index = index);

        LR1_automata.merge_sets();

        LR1_automata.iter().for_each(|state| {
            self.LR1_automata.intern(state.borrow().clone());
        });
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

        let mut action: IndexMap<StateId, IndexMap<SymbolId, Action>> = IndexMap::new();

        let mut goto: IndexMap<StateId, IndexMap<SymbolId, StateId>> = IndexMap::new();

        for state in self.LR1_automata.vec.iter() {
            let state_id = self.LR1_automata.reverse_lookup(state).unwrap();
            for item in state.items.iter() {
                let next_symbol = item.next_symbol(&self.grammar.productions);
                if next_symbol.is_none() {
                    if item.production.ne(&AUGMENT_PRODUCTION_ID) {
                        let mut map = IndexMap::new();
                        item.lookaheads.iter().for_each(|lookahead| {
                            map.insert(*lookahead, Action::REDUCE(item.production.clone()));
                        });

                        action.entry(state_id).insert_entry(map);
                    } else {
                        action
                            .entry(state_id)
                            .insert_entry(IndexMap::from([(EOF_SYMBOL_ID, Action::ACCEPT)]));
                    }
                    continue;
                }
                let symbol = next_symbol.unwrap();
                let mut item_ = item.clone();
                item_.advance_cursor();
                let item_goto_state = state.outgoing.get(&symbol);
                if item_goto_state.is_none() {
                    continue;
                }
                let item_goto_state = item_goto_state.unwrap();
                let goto_state_id = self
                    .LR1_automata
                    .reverse_lookup(&*item_goto_state.borrow())
                    .unwrap();
                if self.grammar.symbols.terminal(&symbol) {
                    action
                        .entry(state_id)
                        .and_modify(|map| {
                            map.insert(symbol.clone(), Action::SHIFT(goto_state_id));
                        })
                        .or_insert(IndexMap::from([(
                            symbol.clone(),
                            Action::SHIFT(goto_state_id),
                        )]));
                }
                if self.grammar.symbols.non_terminal(&symbol) {
                    goto.entry(state_id)
                        .and_modify(|map| {
                            map.insert(symbol, goto_state_id);
                        })
                        .or_insert(IndexMap::from([(symbol, goto_state_id)]));
                }
            }
        }
        self.action = action;
        self.goto = goto;
    }

    //LR-Parsing Algorithm
    // 𝐈𝐍𝐏𝐔𝐓 : An input string 𝑤 and LR-parsing table with functions
    // 𝐴𝐶𝑇𝐼𝑂𝑁 and 𝐺𝑂𝑇𝑂 for a grammar 𝐺
    //
    // 𝐎𝐔𝐓𝐏𝐔𝐓 : If 𝑤 is not in 𝐿(𝐺), an error result
    //
    // 𝐌𝐄𝐓𝐇𝐎𝐃 : Initially, the parser has 𝑆₀ on its stack, where 𝑆₀
    // is the intial state, and 𝑤$ in the input buffer.The parser then
    // executes the following program
    // ------program------
    // let 𝑎 be the first symbol of 𝑤$;
    // while (1) {
    //      let 𝑠 be the state on top of the stack;
    //      if ( 𝐴𝐶𝑇𝐼𝑂𝑁[𝑠,𝑎] = shift 𝑡 ) {
    //          push 𝑡 onto the stack;
    //          let 𝑎 be the next input symbol'
    //      } else if ( 𝐴𝐶𝑇𝐼𝑂𝑁[𝑠,𝑎] = reduce 𝐴 → 𝛽) {
    //          pop |𝛽| symbols off the stack;
    //          let state 𝑡 now be on top of the stack;
    //          push 𝐺𝑂𝑇𝑂[𝑡,𝑎] on to the stack;
    //      } else if (  𝐴𝐶𝑇𝐼𝑂𝑁[𝑠,𝑎] = accecpt ) {
    //          break
    //      } else {
    //          call error-recovery routine;
    //      }
    // }
    pub fn parse(
        &mut self,
        tokens_input: Vec<Token>,
        errors: &mut Vec<ParseError<Token>>,
        ast: &mut AST,
    ) {
        let mut stack: Vec<StateId> = vec![];
        let mut input_iter = tokens_input.iter();
        let mut current_input = if let Some(input) = input_iter.next() {
            input
        } else {
            exit(1)
        };
        let mut previous_input = current_input;
        let mut current_input_symbol = Symbol::TERMINAL(current_input.to_string());
        let mut S0 = self.LR1_automata.map.first().unwrap().1;
        let mut translator_stack: Vec<TranslatorStack> = Vec::new();
        let mut input_token_stack: Vec<Token> = Vec::new();

        stack.push(*S0);
        loop {
            S0 = stack.last().unwrap();
            //every state will be in action_map so unwrap
            let action_map = self.action.get(S0).unwrap();
            let symbol_id = self.grammar.symbols.reverse_lookup(&current_input_symbol);
            if symbol_id.is_none() {
                panic!("symbol not found");
            }
            if let Some(action) = action_map.get(&symbol_id.unwrap()) {
                match action {
                    Action::SHIFT(stateId) => {
                        stack.push(*stateId);

                        //To maintain current input as a stack helps library user;
                        input_token_stack.push(current_input.clone());

                        previous_input = current_input;
                        current_input = input_iter.next().unwrap();
                        current_input_symbol = Symbol::TERMINAL(current_input.to_string());
                    }
                    Action::REDUCE(productionId) => {
                        let production = self.grammar.productions.lookup(*productionId);
                        match &production.action {
                            Some(action) => (action.as_ref())(
                                ast,
                                &mut input_token_stack,
                                &mut translator_stack,
                                errors,
                            ),
                            None => {}
                        };
                        stack.truncate(stack.len() - production.body_len());
                        let stack_top = stack.last().unwrap();
                        let goto_map = self.goto.get(stack_top).unwrap();
                        let goto_stack = goto_map.get(&production.head);
                        if let Some(goto_stack) = goto_stack {
                            stack.push(*goto_stack);
                        }
                    }
                    Action::ACCEPT => {
                        println!("heyyyyyyy");
                        break;
                    }
                    _ => {}
                }
            } else {
                let error_token = current_input;
                //error recovery
                //implement second method in this paper https://ieeexplore.ieee.org/document/6643853
                //@todo need to optimise
                let s0_state = self.LR1_automata.lookup(*S0);
                let mut error_message = self.counstruct_syntax_error_message(S0);

                let deduced_items = s0_state.transistion_productions(&self.grammar.productions);

                if deduced_items.len() == 1 {
                    let item = deduced_items.first().unwrap();
                    let production = self.grammar.productions.lookup(item.production);
                    if production.error_message.is_some() {
                        error_message = production.error_message.unwrap().clone();
                    }
                }

                if current_input_symbol == Symbol::TERMINAL(String::from("EOF")) {
                    errors.push(ParseError {
                        token: previous_input.clone(),
                        message: error_message.clone(),
                        production_end: true,
                    });
                } else {
                    errors.push(ParseError {
                        token: error_token.clone(),
                        message: error_message,
                        production_end: false,
                    });
                }

                return;

                // let mut deduced_production: Option<Production<AST, Token, TranslatorStack>> = None;
                // loop {
                //     stack.pop();
                //     let so_o = stack.last();
                //     if let None = so_o {
                //         break;
                //     }
                //     S0 = so_o.unwrap();
                //     let goto_map = self.goto.get(S0).unwrap();
                //     let keys: Vec<SymbolId> = goto_map.clone().into_keys().collect();
                //     let mut contains = false;
                //     for item in deduced_items.iter() {
                //         let production = self.grammar.productions.lookup(item.production);
                //         if keys.contains(&production.head) {
                //             contains = true;
                //             deduced_production = Some(production);
                //             break;
                //         }
                //     }
                //     if contains {
                //         break;
                //     }
                // }
                // //skip input till input character contains in followset of ...
                // //top_state transition symbol
                // if let None = deduced_production {
                //     break;
                // }
                // let error_production_follow_set = self
                //     .follow_set
                //     .get(&deduced_production.clone().unwrap().head.clone())
                //     .unwrap();
                // loop {
                //     let symbol_id = self.grammar.symbols.reverse_lookup(&current_input_symbol);
                //     if symbol_id.is_none() {
                //         panic!("symbol not found");
                //     }
                //     if error_production_follow_set.contains(&symbol_id.unwrap()) {
                //         if deduced_production.clone().unwrap().error_message.is_some() {
                //             //let a = deduced_production.unwrap().clone(); todo
                //             error_message =
                //                 deduced_production.clone().unwrap().error_message.unwrap();
                //         }
                //         if input_symbol_skip_count == 0 {
                //             errors.push(ParseError {
                //                 token: previous_input.clone(),
                //                 message: error_message,
                //                 production_end: true,
                //             });
                //         } else {
                //             errors.push(ParseError {
                //                 token: error_token.clone(),
                //                 message: error_message,
                //                 production_end: false,
                //             });
                //         }
                //         break;
                //     } else {
                //         input_symbol_skip_count += 1;
                //         previous_input = current_input;
                //         let ci_o = input_iter.next();
                //         if let None = ci_o {
                //             break;
                //         }
                //         current_input = ci_o.unwrap();
                //         current_input_symbol = Symbol::TERMINAL(current_input.to_string());
                //     }
                // }
            }
        }
    }

    fn counstruct_syntax_error_message(&self, state: &StateId) -> String {
        let action_map = self.action.get(state).unwrap();
        let keys: Vec<Symbol> = action_map
            .clone()
            .into_keys()
            .into_iter()
            .map(|symbol_id| self.grammar.symbols.lookup(symbol_id))
            .collect();
        let action_keys: Vec<String> = keys.iter().map(|symbol| symbol.to_string()).collect();
        String::from("Expected ") + join_either_or(action_keys).as_str()
    }
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
