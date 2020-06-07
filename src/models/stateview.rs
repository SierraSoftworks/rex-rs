
pub trait StateView<T> : Into<T> + From<T> {
    fn from_state(state: &T) -> Self;
    fn to_state(&self) -> T;
}