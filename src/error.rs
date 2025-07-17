#[derive(Debug, Clone)]
pub struct ParseError<T> {
    pub token: T,
    pub message: String,
    //if the error is at the end of production set true
    pub production_end: bool,
}
