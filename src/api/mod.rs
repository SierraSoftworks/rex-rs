pub trait StateView<T> {
    fn from_state(state: &T) -> Self;
    fn to_state(&self) -> T;
}