pub trait Interner {
    type T;
    type Id;
    fn intern(&mut self, t: Self::T) -> Self::Id;
    #[must_use]
    #[allow(unused)]
    fn lookup(&self, id: Self::Id) -> Self::T;
    #[must_use]
    fn reverse_lookup(&self, t: &Self::T) -> Option<Self::Id>;
}
