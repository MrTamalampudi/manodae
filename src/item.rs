use crate::{production::Production, symbol::Symbol};

#[derive(Clone, Debug, PartialEq)]
pub struct Item<'a, AST, Token, TranslatorStack> {
    pub production: &'a Production<AST, Token, TranslatorStack>,
    pub cursor: u8,
    pub lookaheads: Vec<Symbol>,
}

impl<'a, AST, Token, TranslatorStack> Item<'a, AST, Token, TranslatorStack> {
    pub fn next_symbol(&self) -> Option<&Symbol> {
        if self.cursor == self.production.body.len() as u8 {
            None
        } else {
            self.production.body.get(self.cursor as usize)
        }
    }
    pub fn advance_cursor(&mut self) {
        self.cursor += 1;
    }
    pub fn is_augment_item(&self) -> bool {
        self.production.head == String::from("S'")
    }
}
