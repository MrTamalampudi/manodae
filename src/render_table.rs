use std::iter::once;

use indexmap::IndexMap;
use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use crate::{parser::LR1_Parser, symbol::Symbol};

///Prettyprints table:
/// 1. Productions
/// 2. Actions
/// 3. Goto
pub fn render<AST, Token, TranslatorStack>(_parser: &LR1_Parser<AST, Token, TranslatorStack>) {
    let mut terminals: Vec<&Symbol> = vec![];
    let mut non_terminals: Vec<&Symbol> = vec![];

    _parser.symbols.iter().for_each(|symbol| {
        if symbol.is_terminal() {
            terminals.push(symbol);
        } else {
            non_terminals.push(symbol);
        }
    });

    let mut action_table = Builder::new();
    let mut goto_table = Builder::new();
    let mut productions_table = Builder::new();

    action_table.push_record(terminals.clone());
    goto_table.push_record(non_terminals.clone());
    productions_table.push_record(["Productions"]);

    _parser.productions.iter().for_each(|prod| {
        productions_table.push_record([format!("{:<25} -> {}", prod.head, {
            prod.body
                .iter()
                .map(|symbol| symbol.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        })])
    });

    productions_table.insert_column(
        0,
        once(String::new()).chain(
            _parser
                .productions
                .iter()
                .map(|prod| prod.index.to_string()),
        ),
    );

    let mut action_state: IndexMap<&Symbol, String> = IndexMap::new();
    terminals.iter().for_each(|symbol| {
        action_state.insert(*symbol, String::new());
    });

    let mut goto_state: IndexMap<&Symbol, String> = IndexMap::new();
    non_terminals.iter().for_each(|symbol| {
        goto_state.insert(*symbol, String::new());
    });

    _parser.LR1_automata.iter().for_each(|state| {
        //Action table
        action_state.values_mut().for_each(|f| f.clear());
        let a_state = _parser.action.get(state);
        if let Some(_a_state) = a_state {
            _a_state.iter().for_each(|(symbol, action)| {
                if symbol.is_terminal() {
                    action_state.entry(symbol).and_modify(|a| {
                        use crate::action::Action::*;
                        match action {
                            SHIFT(state) => a.push_str(format!("S {}", state.index).as_str()),
                            REDUCE(prod) => a.push_str(format!("R {}", prod.index).as_str()),
                            ACCEPT => a.push_str("ACCEPT"),
                            _ => {}
                        }
                    });
                }
            });
        };
        action_table.push_record(action_state.values());

        //GOTO table
        goto_state.values_mut().for_each(|f| f.clear());
        let g_state = _parser.goto.get(state);
        if let Some(_g_state) = g_state {
            _g_state.iter().for_each(|(symbol, state)| {
                if symbol.is_non_terminal() {
                    goto_state
                        .entry(symbol)
                        .and_modify(|b| b.push_str(format!("S {}", state.index).as_str()));
                }
            });
        }
        goto_table.push_record(goto_state.values());
    });

    let index = once(String::new()).chain(_parser.LR1_automata.iter().map(|i| i.index.to_string()));
    action_table.insert_column(0, index.clone());
    goto_table.insert_column(0, index);

    let mut a_table = action_table.build();
    a_table.with(Style::rounded());
    a_table.with(Alignment::center());

    let mut g_table = goto_table.build();
    g_table.with(Style::rounded());
    g_table.with(Alignment::center());

    let mut p_table = productions_table.build();
    p_table.with(Style::rounded());
    p_table.with(Alignment::left());

    println!("states len {}", _parser.LR1_automata.len());

    println!("{p_table}");

    render_states(_parser);

    println!("Action");
    println!("{a_table}");

    println!("GOTO");
    println!("{g_table}");
}

fn render_states<AST, Token, TranslatorStack>(_parser: &LR1_Parser<AST, Token, TranslatorStack>) {
    for state in _parser.LR1_automata.iter() {
        let mut state_builder = Builder::new();
        let mut symbol_builder = Builder::new();
        state_builder.push_record([
            " ".to_string(),
            format!("State {}", state.index.to_string()),
        ]);
        symbol_builder.push_record(["Symbol", "State"]);
        for item in state.items.iter() {
            let head = &item.production.head;
            let mut body_1 = vec![];
            let mut body_2 = vec![];
            item.production
                .body
                .iter()
                .enumerate()
                .for_each(|(index, symbol)| {
                    if index < (item.cursor as usize) {
                        body_1.push(symbol.to_string());
                    } else {
                        body_2.push(symbol.to_string());
                    }
                });
            state_builder.push_record([
                head,
                &format!(
                    "{} . {} / {}",
                    body_1.join(" "),
                    body_2.join(" "),
                    item.lookaheads
                        .iter()
                        .map(|la| la.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ]);
        }
        for (symbol, state_) in state.outgoing.iter() {
            symbol_builder.push_record([symbol.to_string(), state_.borrow().index.to_string()]);
        }

        let mut a_table = state_builder.build();
        a_table.with(Style::rounded());
        let mut s_table = symbol_builder.build();
        s_table.with(Style::rounded());

        println!("-------- State {} --------", state.index);
        println!("{}", a_table);
        println!("{}", s_table);
    }
}
