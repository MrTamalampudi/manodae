use crate::{production::ProductionId, state::StateId};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SHIFT(StateId),
    REDUCE(ProductionId),
    ACCEPT,
    ERROR(String),
}
