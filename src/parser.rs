use std::{cell::RefCell, fmt::Debug, ops::Deref, process::exit, rc::Rc, time::Instant};

use indexmap::{IndexMap, IndexSet};

use crate::{
    action::Action,
    error::ParseError,
    first::compute_first_set,
    follow::compute_follow_set,
    grammar::Grammar,
    item::{Item, ItemVecExtension},
    production::Production,
    state::{State, StateVecExtension},
    symbol::Symbol,
};

const AUGMENTED_PRODUCTION_HEAD: &'static str = "S'";

#[derive(Debug, Clone)]
pub struct LR1_Parser<AST, Token, TranslatorStack> {
    pub grammar: Grammar<AST, Token, TranslatorStack>,
    pub LR1_automata: Vec<Rc<State<AST, Token, TranslatorStack>>>,
    pub follow_set: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>>,
    pub first_set: IndexMap<Rc<Symbol>, IndexSet<Rc<Symbol>>>,
    pub conflicts: bool,
    pub goto: IndexMap<
        Rc<State<AST, Token, TranslatorStack>>,
        IndexMap<Rc<Symbol>, Rc<State<AST, Token, TranslatorStack>>>,
    >,
    pub action: IndexMap<
        Rc<State<AST, Token, TranslatorStack>>,
        IndexMap<Rc<Symbol>, Action<AST, Token, TranslatorStack>>,
    >,
}

impl<AST, Token, TranslatorStack> LR1_Parser<AST, Token, TranslatorStack>
where
    AST: Clone + Debug + PartialEq,
    Token: ToString + Debug + Clone + PartialEq,
    TranslatorStack: Clone + Debug + PartialEq,
{
    pub fn new(
        grammar: Grammar<AST, Token, TranslatorStack>,
    ) -> LR1_Parser<AST, Token, TranslatorStack> {
        //collect all grammar symbols without duplicates

        let first_set = compute_first_set(&grammar);
        let follow_set = compute_follow_set(&grammar);

        let mut production_head_map: IndexMap<
            String,
            IndexSet<Rc<Production<AST, Token, TranslatorStack>>>,
        > = IndexMap::new();

        grammar.productions.iter().for_each(|production| {
            production_head_map
                .entry(production.head.clone())
                .and_modify(|entry| {
                    entry.insert(Rc::clone(production));
                })
                .or_insert(IndexSet::from([Rc::clone(production)]));
        });

        let mut a = LR1_Parser {
            grammar,
            LR1_automata: Vec::new(),
            first_set,
            follow_set,
            conflicts: false,
            action: IndexMap::new(),
            goto: IndexMap::new(),
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
    fn clousure(&self, items: &mut Vec<Item<AST, Token, TranslatorStack>>) {
        let mut items_count = 0;
        let mut items_iterated_count = 0;
        while items.len().ne(&items_count) {
            items_count = items.len();
            let mut new_items: Vec<Item<AST, Token, TranslatorStack>> = Vec::new();
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
                let b_productions: &IndexSet<_> = self
                    .grammar
                    .production_head_map
                    .get(&production_B.to_string())
                    .unwrap();
                let mut lookaheads = Vec::new();
                for first in first_of.iter() {
                    for terminal_b in self.first_set.get(first).unwrap().iter() {
                        lookaheads.push(Rc::clone(terminal_b));
                    }
                }
                for b_production in b_productions.iter() {
                    let p = b_production.deref();
                    let item_ = Item {
                        production: Rc::new(p.clone()),
                        cursor: 0,
                        lookaheads: lookaheads.clone(),
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
        state_: &Rc<RefCell<State<AST, Token, TranslatorStack>>>,
        symbol: &Rc<Symbol>,
    ) -> Rc<RefCell<State<AST, Token, TranslatorStack>>> {
        let mut new_items = vec![];
        for item in state_.borrow().items.iter() {
            let item_symbol = item.next_symbol();
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
        let transition_productions = new_items.clone();
        self.clousure(&mut new_items);
        let state = State::new(
            0,
            new_items,
            transition_productions,
            IndexMap::new(),
            vec![],
        );
        Rc::new(RefCell::new(state))
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
            production: Rc::clone(self.grammar.productions.first().unwrap()),
            cursor: 0,
            lookaheads: vec![Rc::new(Symbol::TERMINAL(String::from("EOF")))],
        };
        let mut S0_items = vec![augmented_item];
        self.clousure(&mut S0_items);
        let mut LR1_automata = vec![Rc::new(RefCell::new(State {
            transition_productions: vec![],
            index: 0,
            items: S0_items,
            outgoing: IndexMap::new(),
            incoming: vec![],
        }))];
        let mut states_count = 0;
        let mut states_iterated_count = 0;
        let mut symbols = vec![];
        symbols.extend(&self.grammar.non_terminals);
        symbols.extend(&self.grammar.terminals);
        while LR1_automata.len().ne(&states_count) {
            states_count = LR1_automata.len();
            let mut new_state = vec![];
            for states_index in states_iterated_count..LR1_automata.len() {
                let state = LR1_automata.get(states_index).unwrap();
                let mut goto_map = IndexMap::new();
                for symbol in symbols.iter() {
                    let goto_productions_state = self.goto(&state, symbol);
                    let existing_state = LR1_automata.custom_get(&goto_productions_state);
                    if !goto_productions_state.borrow().items.is_empty() && existing_state.is_none()
                    {
                        goto_map.insert(Rc::clone(symbol), Rc::clone(&goto_productions_state));
                        new_state.push(Rc::clone(&goto_productions_state));
                    }
                    if let Some(e_state) = existing_state {
                        goto_map.insert(Rc::clone(symbol), Rc::clone(&e_state));
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

        self.LR1_automata = LR1_automata
            .iter()
            .map(|state| Rc::new(state.borrow().clone()))
            .collect();
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

        let mut action: IndexMap<
            Rc<State<AST, Token, TranslatorStack>>,
            IndexMap<Rc<Symbol>, Action<AST, Token, TranslatorStack>>,
        > = IndexMap::new();

        let mut goto: IndexMap<
            Rc<State<AST, Token, TranslatorStack>>,
            IndexMap<Rc<Symbol>, Rc<State<AST, Token, TranslatorStack>>>,
        > = IndexMap::new();

        let mut transition_prod_map: IndexMap<
            Item<AST, Token, TranslatorStack>,
            &Rc<State<AST, Token, TranslatorStack>>,
        > = IndexMap::new();

        self.LR1_automata.iter().for_each(|state| {
            for item in state.transition_productions.iter() {
                transition_prod_map.entry(item.clone()).or_insert(state);
            }
        });

        for state in self.LR1_automata.iter() {
            for item in state.items.iter() {
                let next_symbol = item.next_symbol();
                if next_symbol.is_none() {
                    if item
                        .production
                        .head
                        .ne(&AUGMENTED_PRODUCTION_HEAD.to_string())
                    {
                        let mut map = IndexMap::new();
                        item.lookaheads.iter().for_each(|lookahead| {
                            map.insert(lookahead.clone(), Action::REDUCE(item.production.clone()));
                        });
                        action.entry(state.clone()).insert_entry(map);
                    } else {
                        action.entry(state.clone()).insert_entry(IndexMap::from([(
                            Rc::new(Symbol::TERMINAL(String::from("EOF"))),
                            Action::ACCEPT,
                        )]));
                    }
                    continue;
                }
                let symbol = next_symbol.unwrap();
                let mut item_ = item.clone();
                item_.advance_cursor();
                let item_goto_state = state.outgoing.get(symbol);
                if item_goto_state.is_none() {
                    continue;
                }
                let item_goto_state = item_goto_state.unwrap();
                if let Symbol::TERMINAL(_) = symbol.deref() {
                    action
                        .entry(state.clone())
                        .and_modify(|map| {
                            map.insert(
                                symbol.clone(),
                                Action::SHIFT(Rc::new(item_goto_state.borrow().clone())),
                            );
                        })
                        .or_insert(IndexMap::from([(
                            symbol.clone(),
                            Action::SHIFT(Rc::new(item_goto_state.borrow().clone())),
                        )]));
                }
                if let Symbol::NONTERMINAL(_) = symbol.deref() {
                    goto.entry(state.clone())
                        .and_modify(|map| {
                            map.insert(symbol.clone(), Rc::new(item_goto_state.borrow().clone()));
                        })
                        .or_insert(IndexMap::from([(
                            symbol.clone(),
                            Rc::new(item_goto_state.borrow().clone()),
                        )]));
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
        let mut stack: Vec<Rc<State<AST, Token, TranslatorStack>>> = vec![];
        let mut input_iter = tokens_input.iter();
        let mut current_input = if let Some(input) = input_iter.next() {
            input
        } else {
            exit(1)
        };
        let mut previous_input = current_input;
        let mut current_input_symbol = Symbol::TERMINAL(current_input.to_string());
        let mut S0 = self.LR1_automata.first().unwrap();
        let mut translator_stack: Vec<TranslatorStack> = Vec::new();
        let mut input_token_stack: Vec<Token> = Vec::new();

        stack.push(Rc::clone(S0));
        loop {
            S0 = stack.last().unwrap();
            //every state will be in action_map so unwrap
            let action_map = self.action.get(S0).unwrap();
            if let Some(action) = action_map.get(&current_input_symbol) {
                match action {
                    Action::SHIFT(state) => {
                        stack.push(Rc::clone(state));

                        //To maintain current input as a stack helps library user;
                        input_token_stack.push(current_input.clone());

                        previous_input = current_input;
                        current_input = input_iter.next().unwrap();
                        current_input_symbol = Symbol::TERMINAL(current_input.to_string());
                    }
                    Action::REDUCE(production) => {
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
                        let goto_stack =
                            goto_map.get(&Symbol::NONTERMINAL(production.head.to_string()));
                        if let Some(goto_stack) = goto_stack {
                            stack.push(Rc::clone(goto_stack));
                        }
                    }
                    Action::ACCEPT => {
                        println!("heyyyyyyy");
                        break;
                    }
                    _ => {}
                }
            } else {
                let mut input_symbol_skip_count = 0;
                let error_token = current_input;
                //error recovery
                //implement second method in this paper https://ieeexplore.ieee.org/document/6643853
                //@todo need to optimise
                let mut error_message = self.counstruct_syntax_error_message(S0);

                let deduced_items = S0.transition_productions.clone();
                let mut deduced_production: Option<Rc<Production<AST, Token, TranslatorStack>>> =
                    None;
                loop {
                    stack.pop();
                    let so_o = stack.last();
                    if let None = so_o {
                        break;
                    }
                    S0 = so_o.unwrap();
                    let goto_map = self.goto.get(S0).unwrap();
                    let keys: Vec<Rc<Symbol>> = goto_map.clone().into_keys().collect();
                    let mut contains = false;
                    for item in deduced_items.iter() {
                        if keys
                            .contains(&Rc::new(Symbol::NONTERMINAL(item.production.head.clone())))
                        {
                            contains = true;
                            deduced_production = Some(item.production.clone());
                            break;
                        }
                    }
                    if contains {
                        break;
                    }
                }
                //skip input till input character contains in followset of ...
                //top_state transition symbol
                if let None = deduced_production {
                    break;
                }
                let error_production_follow_set = self
                    .follow_set
                    .get(&Symbol::NONTERMINAL(
                        deduced_production.clone().unwrap().head.clone(),
                    ))
                    .unwrap();
                loop {
                    if error_production_follow_set.contains(&current_input_symbol) {
                        if deduced_production.clone().unwrap().error_message.is_some() {
                            //let a = deduced_production.unwrap().clone(); todo
                            error_message = "SOme".to_string();
                        }
                        if input_symbol_skip_count == 0 {
                            errors.push(ParseError {
                                token: previous_input.clone(),
                                message: error_message,
                                production_end: true,
                            });
                        } else {
                            errors.push(ParseError {
                                token: error_token.clone(),
                                message: error_message,
                                production_end: false,
                            });
                        }
                        break;
                    } else {
                        input_symbol_skip_count += 1;
                        previous_input = current_input;
                        let ci_o = input_iter.next();
                        if let None = ci_o {
                            break;
                        }
                        current_input = ci_o.unwrap();
                        current_input_symbol = Symbol::TERMINAL(current_input.to_string());
                    }
                }
            }
        }
    }

    fn counstruct_syntax_error_message(
        &self,
        state: &State<AST, Token, TranslatorStack>,
    ) -> String {
        let action_map = self.action.get(state).unwrap();
        let keys: Vec<Rc<Symbol>> = action_map.clone().into_keys().collect();
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
