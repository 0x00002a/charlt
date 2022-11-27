pub trait Holds {
    type Item;
    fn map<O, T, F: FnOnce(Self::Item) -> T>(self, f: F) -> O
    where
        O: Holds<Item = T>;
    fn into_value(self) -> Self::Item;
}
