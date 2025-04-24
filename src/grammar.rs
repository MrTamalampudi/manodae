//cfg grammar should be in bnf format

use std::fmt::Debug;

use crate::{production::Production, terminal::Terminal};

#[derive(Debug)]
pub struct Grammar<T>
where
    T: Clone + PartialEq + Eq + Debug + Terminal<T>,
{
    pub productions: Vec<Production<T>>,
}

impl<T: Clone + Debug + PartialEq + Eq + Terminal<T>> Grammar<T> {
    pub fn new() -> Grammar<T> {
        Grammar {
            productions: Vec::new(),
        }
    }
}

#[macro_export]
macro_rules! grammar {
    (
        $terminal_type:ident,
        $($head:ident -> $($terminal:path $([$non_terminal:ident])*)|+);*
    ) => {{
        let mut grammar: Grammar<$terminal_type> = Grammar::new();
        $({
            $({let mut body_ : Vec<Symbol<$terminal_type>> = Vec::new();
            body_.push(Symbol::TERMINAL($terminal));
            $(
                body_.push(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
            )*
            let production:Production<$terminal_type> = Production {
                head: stringify!($head).to_string(),
                body: body_,
                cursor_pos: 0,
                index: grammar.productions.len() + 1
            };
            grammar.productions.push(production);})+
        })+
        grammar
    }}
}
