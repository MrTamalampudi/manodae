#[derive(Clone, Debug)]
pub enum Token {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    EOF,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::A => String::from("A"),
            Token::B => String::from("B"),
            Token::C => String::from("C"),
            Token::D => String::from("D"),
            Token::E => String::from("E"),
            Token::F => String::from("F"),
            Token::G => String::from("G"),
            Token::H => String::from("H"),
            Token::EOF => String::from("EOF"),
        }
    }
}
