#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SHIFT(usize),
    REDUCE(usize),
    ACCEPT,
    ERROR(String),
}
