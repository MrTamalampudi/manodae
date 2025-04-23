//cfg grammar should be in bnf format

#[macro_export]
macro_rules! grammar {
    (
        $($head:ident -> $($terminal:path $([$non_terminal:ident])*)|+);*
    ) => {{
        let mut grammar: Grammar = Grammar::new();
        $({
            $({let mut body_ : Vec<Symbol> = Vec::new();
            body_.push(Symbol::TERMINAL($terminal));
            $(
                body_.push(Symbol::NONTERMINAL(stringify!($non_terminal).to_string()));
            )*
            let production:Production = Production {
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
