//cfg grammar should be in bnf format

use std::fmt::Debug;

use crate::production::Production;

#[derive(Debug)]
pub struct Grammar {
    pub productions: Vec<Production>,
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            productions: Vec::new(),
        }
    }
}

#[macro_export]
macro_rules! grammar {
    (
        $terminal_type:ident,
        $(
            $head:ident -> $(
                $([$($terminal:expr),*])? $($non_terminal:ident)* $({error:$error:literal})?
            )|+
        );+
    ) => {{
        let mut grammar: Grammar= Grammar::new();
        $({
            $({let mut body_ : Vec<Symbol> = Vec::new();
            $($(body_.push(Symbol::TERMINAL($terminal.to_string_c()));)*)?
            $(
                body_.push(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
            )*
            let mut production:Production = Production {
                head: stringify!($head).to_string(),
                body: body_,
                cursor_pos: 0,
                index: grammar.productions.len() + 1,
                error_message: None
            };
            $(
              if $error.to_string().len() > 0 {
                  production.error_message = Some($error.to_string());
              }
            )?
            grammar.productions.push(production);})+
        })+
        grammar
    }}
}
