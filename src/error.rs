#[derive(Debug, Clone)]
pub struct ParseError<Token> {
    pub token: Token,
    pub message: String,
    //if the error is at the end of production set true
    pub production_end: bool,
}

impl<Token> ParseError<Token> {
    pub fn new(token: Token, message: String) -> ParseError<Token> {
        ParseError {
            token,
            message,
            production_end: false,
        }
    }
}
