use std::ops::Range;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Range<usize>,
    pub message: String,
    //if the error is at the end of production set true
    pub production_end: bool,
}

impl ParseError {
    pub fn new(span: Range<usize>, message: String) -> ParseError {
        ParseError {
            span,
            message,
            production_end: false,
        }
    }
}
