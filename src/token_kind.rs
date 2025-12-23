use std::fmt::Debug;

pub trait TokenKind: ToString + Debug + Clone {
    type TokenKind;
    ///returns an developer intended error token which is used while parsing
    fn error() -> Self::TokenKind;
    ///returns an developer intended end of the file token
    fn eof() -> Self::TokenKind;
}
