use std::rc::Rc;

use crate::{production::ProductionId, state::State};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SHIFT(Rc<State>),
    REDUCE(ProductionId),
    ACCEPT,
    ERROR(String),
}
