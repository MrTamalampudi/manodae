use std::{collections::HashMap, fmt::Debug};

use crate::{action::Action, production::Production, symbol::Symbol, terminal::Terminal};

#[derive(Debug, Clone)]
pub struct State<T>
where
    T: Clone + PartialEq + Eq + Debug + Terminal<T>,
{
    pub state: usize,
    pub productions: Vec<Production<T>>,
    pub transition_symbol: Symbol<T>,
    pub action: HashMap<T, Action>,
    pub goto: HashMap<String, usize>,
}
