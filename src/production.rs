use std::fmt::Debug;

use crate::{terminal::Terminal, Symbol};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Production<T>
where
    T: Clone + Debug + PartialEq + Eq + Terminal,
{
    pub head: String,
    pub body: Vec<Symbol<T>>,
    pub cursor_pos: usize,
    pub index: usize,
}

impl<T: Clone + Debug + PartialEq + Eq + Terminal> Production<T> {
    pub fn next_symbol(&self) -> Option<&Symbol<T>> {
        if self.cursor_pos == self.body.len() {
            None
        } else {
            self.body.get(self.cursor_pos)
        }
    }
    pub fn advance_cursor(&mut self) {
        self.cursor_pos += 1;
    }
}
