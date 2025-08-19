//cfg grammar should be in bnf format

use crate::production_LR1::Production_LR1;

pub type Grammar<AST, Token, TranslatorStack> = Vec<Production_LR1<AST, Token, TranslatorStack>>;

#[macro_export]
macro_rules! grammar {
    (
        $(
            $head:ident -> $(
                $([$($terminal:expr),*])?
                $($non_terminal:ident)*
                $({error:$error:literal})?
                $({action:|$arg1:ident,$arg2:ident,$arg3:ident,$arg4:ident| $expr:block})?
            )|+
        );+;
    ) => {{
        let mut grammar = Vec::new();
        let augmented_production = Production_LR1 {
            head: String::from("S'"),
            body: vec![Symbol::NONTERMINAL(String::from("Start"))],
            error_message: None,
            #[allow(unused_variables)]
            action: Some(Arc::new(|ast, token_stack, tl_stack, errors| {})),
        };
        grammar.push(augmented_production);
        $({
            $({let mut body_ : Vec<Symbol> = Vec::new();
            $($(body_.push(Symbol::TERMINAL($terminal.to_string()));)*)?
            $(
                body_.push(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
            )*
            #[allow(unused_mut)]
            let mut production = Production_LR1 {
                head: stringify!($head).to_string(),
                body: body_,
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
            grammar.push(production);})+
        })+
        grammar
    }}
}
