//cfg grammar should be in bnf format

use std::{fmt::Debug, rc::Rc};

use indexmap::{IndexMap, IndexSet};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{production::Production, symbol::Symbol};

#[derive(Debug, Clone)]
pub struct Grammar<AST, Token, TranslatorStack> {
    pub non_terminals: IndexSet<Rc<Symbol>>,
    pub terminals: IndexSet<Rc<Symbol>>,
    pub start: Rc<Symbol>,
    pub productions: IndexSet<Rc<Production<AST, Token, TranslatorStack>>>,
    //used only when constructing table, no need for parsing
    pub production_head_map:
        IndexMap<String, IndexSet<Rc<Production<AST, Token, TranslatorStack>>>>,
}

impl<AST, Token, TranslatorStack> Grammar<AST, Token, TranslatorStack> {
    pub fn new() -> Self {
        Grammar {
            non_terminals: IndexSet::new(),
            terminals: IndexSet::new(),
            start: Rc::new(Symbol::NONTERMINAL("Start".to_string())),
            productions: IndexSet::new(),
            production_head_map: IndexMap::new(),
        }
    }
}

#[macro_export]
macro_rules! grammar {
    (
        $(
            $head:ident -> $(
                $([$terminal:expr])?
                $($non_terminal:ident)*
                $({error:$error:literal})?
                $({action:|$arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident| $expr:block})?
            )|+
        );+;
    ) => {{
        let mut grammar = Grammar::new();
        let mut non_terminals = indexmap::IndexSet::new();
        let mut terminals = indexmap::IndexSet::new();
        let augmented_production = Production {
            head: String::from("S'"),
            body: vec![std::rc::Rc::new(Symbol::NONTERMINAL(String::from("Start")))],
            error_message: None,
            #[allow(unused_variables)]
            action: Some(Rc::new(|ast, token_stack, tl_stack, errors| {})),
            action_tokens : quote::quote!{Rc::new(|$arg1,$arg2,$arg3,$arg4| {})},
            index: 0
        };
        grammar.productions.insert(std::rc::Rc::new(augmented_production));
        $({
            $({let mut body_ : Vec<std::rc::Rc<Symbol>> = Vec::new();
                $(
                    let terminal_ref = std::rc::Rc::new(Symbol::TERMINAL($terminal.to_string()));
                    terminals.insert(std::rc::Rc::clone(&terminal_ref));
                    body_.push(std::rc::Rc::clone(&terminal_ref));
                )?
                $(
                    let non_terminal_ref = std::rc::Rc::new(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
                    non_terminals.insert(std::rc::Rc::clone(&non_terminal_ref));
                    body_.push(std::rc::Rc::clone(&non_terminal_ref));
                )*
            #[allow(unused_mut)]
            let mut production = Production {
                head: stringify!($head).to_string(),
                body: body_,
                error_message: None,
                action:None,
                action_tokens : quote::quote!{Rc::new(|$arg1,$arg2,$arg3,$arg4| {})},
                index: grammar.productions.len()
            };
            $(
              if $error.to_string().len() > 0 {
                  production.error_message = Some($error.to_string());
              }
            )?
            $(
                {production.action = Some(Rc::new(|$arg1,$arg2,$arg3,$arg4| $expr))}
                {production.action_tokens = quote::quote!{Rc::new(|$arg1,$arg2,$arg3,$arg4| $expr)}}
            )?
            grammar.productions.insert(std::rc::Rc::new(production));})+
        })+
        grammar.terminals = terminals;
        grammar.non_terminals = non_terminals;
        grammar
    }}
}
