pub trait Terminal {
    fn get_ending_token() -> String;
    fn to_string(&self) -> String {
        String::new()
    }
}
