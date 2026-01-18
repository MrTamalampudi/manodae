//cfg grammar should be in bnf format

use std::{fmt::Debug, hash::Hash};

use indexmap::{IndexMap, IndexSet};

use crate::{
    production::{ProductionId, Productions},
    symbol::{SymbolId, Symbols, START_SYMBOL_ID},
    token::TokenKind,
};

#[derive(Debug, Clone)]
pub struct Grammar<AST, Token, TranslatorStack> {
    pub symbols: Symbols,
    pub start: SymbolId,
    pub productions: Productions<AST, Token, TranslatorStack>,
    //used only when constructing table, no need for parsing
    pub production_head_map: IndexMap<SymbolId, IndexSet<ProductionId>>,
}

impl<AST, Token, TranslatorStack> Hash for Grammar<AST, Token, TranslatorStack>
where
    Token: TokenKind,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbols.hash(state);
        self.start.hash(state);
        self.productions.hash(state);
    }
}

impl<AST, Token, TranslatorStack> Grammar<AST, Token, TranslatorStack>
where
    AST: Clone,
    Token: Clone + TokenKind,
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

//need to improve lot of repetetion

#[macro_export]
macro_rules! start_production {
    (
        $grammar:ident,
        $($start_non_terminal:ident)+
        $({ |$s_arg1:ident,$s_arg2:ident,$s_arg3:ident,$s_arg4:ident| $s_expr:expr})?
    ) => {
            let mut body_ : Vec<SymbolId> = Vec::new();
            $(
                let non_terminal = Symbol::NONTERMINAL(stringify!($start_non_terminal).to_string());
                let non_terminal_id = $grammar.symbols.intern(non_terminal);
                body_.push(non_terminal_id);
            )+
            #[allow(unused_mut)]
            let mut production = Production {
                head: SymbolId(2),
                body: body_,
                error_message: None,
                action:None,
                action_tokens : quote::quote!{Some(Rc::new(|ast,token_stack,tl_stack,errors| {}))},
                index: $grammar.productions.vec.len()
            };
            $(
                production.action = Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr));
                production.action_tokens = quote::quote!{Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr))};
            )?
            $grammar.productions.intern(production);
    }
}

#[macro_export]
macro_rules! terminal_production {
    (
        $grammar:ident,
        $head:ident,
        [$terminal:expr]
        $({ |$s_arg1:ident,$s_arg2:ident,$s_arg3:ident,$s_arg4:ident| $s_expr:expr})?
    ) => {

        //lhs
        let head = Symbol::NONTERMINAL(stringify!($head).to_string());
        let head_id = $grammar.symbols.intern(head);

        //rhs
        let mut body_ : Vec<SymbolId> = Vec::new();
        let terminal = Symbol::TERMINAL($terminal.to_string());
        let terminal_id = $grammar.symbols.intern(terminal);
        body_.push(terminal_id);

        //production
        #[allow(unused_mut)]
        let mut production = Production {
            head: head_id,
            body: body_,
            error_message: None,
            action:None,
            action_tokens : quote::quote!{Some(Rc::new(|ast,token_stack,tl_stack,errors| {}))},
            index: $grammar.productions.vec.len()
        };

        $(
            production.action = Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr));
            production.action_tokens = quote::quote!{Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr))};
        )?

        $grammar.productions.intern(production);
    };
}

#[macro_export]
macro_rules! non_terminal_production {
    (
        $grammar:ident,
        $head:ident,
        $($non_terminal:ident)+
        $({ |$s_arg1:ident,$s_arg2:ident,$s_arg3:ident,$s_arg4:ident| $s_expr:expr})?
    ) => {

        //lhs
        let head = Symbol::NONTERMINAL(stringify!($head).to_string());
        let head_id = $grammar.symbols.intern(head);

        //rhs
        let mut body_ : Vec<SymbolId> = Vec::new();
        $(
            let non_terminal = Symbol::NONTERMINAL(stringify!($non_terminal).to_string());
            let non_terminal_id = $grammar.symbols.intern(non_terminal);
            body_.push(non_terminal_id);
        )+

        //production
        #[allow(unused_mut)]
        let mut production = Production {
            head: head_id,
            body: body_,
            error_message: None,
            action:None,
            action_tokens : quote::quote!{Some(Rc::new(|ast,token_stack,tl_stack,errors| {}))},
            index: $grammar.productions.vec.len()
        };

        $(
            production.action = Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr));
            production.action_tokens = quote::quote!{Some(Rc::new(|$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr))};
        )?

        $grammar.productions.intern(production);
    };
}

#[macro_export]
macro_rules! grammar{
    (
        Start -> $(
            $($start_non_terminal:ident)+
            $({|$s_arg1:ident,$s_arg2:ident,$s_arg3:ident,$s_arg4:ident| $s_expr:expr})?
        )|+;

        $([non_terminal_productions])?

        $(
            $non_terminal_head:ident -> $(
                $($non_terminal:ident)+
                $({|$n_arg1:ident,$n_arg2:ident,$n_arg3:ident,$n_arg4:ident| $n_expr:expr})?
            )|+
        ;)*

        [terminal_productions]

        $(
            $terminal_head:ident -> $(
                [$end_terminal:expr]
                $({|$e_arg1:ident,$e_arg2:ident,$e_arg3:ident,$e_arg4:ident| $e_expr:expr})?
            )|+;
        )+
    ) => {{
        let mut grammar = Grammar::new();
        //start production
        $(
            $crate::start_production!(
                grammar,
                $($start_non_terminal)+
                $({ |$s_arg1,$s_arg2,$s_arg3,$s_arg4| $s_expr })?
            );
        )+

        //non_terminal_productions
        $(
            $(
                $crate::non_terminal_production!(
                    grammar,
                    $non_terminal_head,
                    $($non_terminal)+
                    $({|$n_arg1,$n_arg2,$n_arg3,$n_arg4| $n_expr})?
                );
            )+
        )?

        //terminal_productions
        $(
            $(
                $crate::terminal_production!(
                    grammar,
                    $terminal_head,
                    [$end_terminal]
                    $({|$e_arg1,$e_arg2,$e_arg3,$e_arg4| $e_expr})?
                );
            )+
        )+

        grammar
    }}
}
