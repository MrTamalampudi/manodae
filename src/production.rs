use std::{fmt::Debug, sync::Arc};

use crate::symbol::Symbol;

#[derive(Clone)]
pub struct Production<T> {
    pub head: String,
    pub body: Vec<Symbol>,
    pub cursor_pos: usize,
    pub index: usize,
    pub error_message: Option<String>,
    pub action: Option<Arc<dyn Fn(&Vec<T>)>>,
}

impl<T> std::fmt::Debug for Production<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Production")
            .field("head", &self.head)
            .field("body", &self.body)
            .field("cursor_pos", &self.cursor_pos)
            .field("index", &self.index)
            .field("error_message", &self.error_message)
            .finish_non_exhaustive()
    }
}

impl<T> PartialEq for Production<T> {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head
            && self.body == other.body
            && self.cursor_pos == other.cursor_pos
            && self.index == other.index
            && self.error_message == other.error_message
    }
}

impl<T> Production<T> {
    pub fn next_symbol(&self) -> Option<&Symbol> {
        if self.cursor_pos == self.body.len() {
            None
        } else {
            self.body.get(self.cursor_pos)
        }
    }
    pub fn advance_cursor(&mut self) {
        self.cursor_pos += 1;
    }
    pub fn is_augment_production(&self) -> bool {
        self.head == String::from("S'")
    }
}
