pub trait Terminal {
    fn get_ending_token() -> String;
    fn to_string_c(&self) -> String;
}
