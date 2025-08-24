pub trait VecExtension<T> {
    fn custom_contains(&self, item_to_find: &T) -> bool;
    #[allow(unused_variables)]
    fn custom_eq(&self, others: Vec<T>) -> bool {
        false
    }
}
