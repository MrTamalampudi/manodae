//cfg grammar should be in bnf format

use std::fmt::Debug;

use crate::production::Production;

#[derive(Debug)]
pub struct Grammar<AST, Token, TranslatorStack> {
    pub productions: Vec<Production<AST, Token, TranslatorStack>>,
}

impl<AST, Token, TranslatorStack> Grammar<AST, Token, TranslatorStack> {
    pub fn new() -> Grammar<AST, Token, TranslatorStack> {
        Grammar {
            productions: Vec::new(),
        }
    }
}

#[macro_export]
macro_rules! grammar {
    (
        $terminal_type:ident,
        $ast_type:ident,
        $token_stack_type:ident,
        $translator_stack_type:ident,
        $(
            $head:ident -> $(
                $([$($terminal:expr),*])?
                $($non_terminal:ident)*
                $({error:$error:literal})?
                $({action:|$arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident| $expr:block})?
            )|+
        );+;
    ) => {{
        let mut grammar: Grammar<$ast_type,$token_stack_type,$translator_stack_type> = Grammar::new();
        $({
            $({let mut body_ : Vec<Symbol> = Vec::new();
            $($(body_.push(Symbol::TERMINAL($terminal.to_string_c()));)*)?
            $(
                body_.push(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
            )*
            let mut production:Production<$ast_type,$token_stack_type,$translator_stack_type> = Production {
                head: stringify!($head).to_string(),
                body: body_,
                cursor_pos: 0,
                index: grammar.productions.len() + 1,
                error_message: None,
                action:None
            };
            $(
              if $error.to_string().len() > 0 {
                  production.error_message = Some($error.to_string());
              }
            )?
            $(
                {production.action = Some(Arc::new(|$arg1,$arg2,$arg3,$arg4| $expr))}
            )?
            grammar.productions.push(production);})+
        })+
        grammar
    }}
}
