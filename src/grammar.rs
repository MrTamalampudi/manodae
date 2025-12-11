//cfg grammar should be in bnf format

use std::fmt::Debug;

use indexmap::{IndexMap, IndexSet};

use crate::{
    production::{ProductionId, Productions},
    symbol::{SymbolId, Symbols, START_SYMBOL_ID},
};

#[derive(Debug, Clone)]
pub struct Grammar<AST, Token, TranslatorStack> {
    pub symbols: Symbols,
    pub start: SymbolId,
    pub productions: Productions<AST, Token, TranslatorStack>,
    //used only when constructing table, no need for parsing
    pub production_head_map: IndexMap<SymbolId, IndexSet<ProductionId>>,
}

impl<AST, Token, TranslatorStack> Grammar<AST, Token, TranslatorStack>
where
    AST: Clone,
    Token: Clone,
    TranslatorStack: Clone,
{
    pub fn new() -> Self {
        Grammar {
            symbols: Symbols::new(),
            start: START_SYMBOL_ID,
            productions: Productions::new(),
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
        $({
            $({let mut body_ : Vec<SymbolId> = Vec::new();
                $(
                    let terminal = Symbol::TERMINAL($terminal.to_string());
                    let terminal_id = grammar.symbols.intern(terminal);
                    body_.push(terminal_id);
                )?
                $(
                    let non_terminal = Symbol::NONTERMINAL(stringify!($non_terminal).to_string());
                    let non_terminal_id = grammar.symbols.intern(non_terminal);
                    body_.push(non_terminal_id);
                )*
            let head = Symbol::NONTERMINAL(stringify!($head).to_string());
            let head_id = grammar.symbols.intern(head);
            #[allow(unused_mut)]
            let mut production = Production {
                head: head_id,
                body: body_,
                error_message: None,
                action:None,
                action_tokens : quote::quote!{c!({})}, //c1 macro expands to Rc::new(|ast,...| {})
                index: grammar.productions.vec.len()
            };
            $(
              if $error.to_string().len() > 0 {
                  production.error_message = Some($error.to_string());
              }
            )?
            $(
                {production.action = Some(Rc::new(|$arg1,$arg2,$arg3,$arg4| $expr))}
                {production.action_tokens = quote::quote!{c!($expr)}}
            )?

            grammar.productions.intern(production);})+
        })+
        grammar
    }}
}
