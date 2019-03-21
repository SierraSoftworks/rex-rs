use std::sync::Arc;
use ::state::Container;
use std::ops::Deref;
use actix_web::{FromRequest, HttpRequest};

pub struct Stator<T: Send + Sync + 'static>(Arc<Container>, std::marker::PhantomData<T>);

impl<T: Send + Sync + 'static> Deref for Stator<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.get()
    }
}

impl<T: Send + Sync + 'static> FromRequest<Arc<Container>> for Stator<T> {
    type Config = ();
    type Result = Stator<T>;

    #[inline]
    fn from_request(req: &HttpRequest<Arc<Container>>, _: &Self::Config) -> Self::Result {
        let state: Arc<Container> = req.state().clone();
        Stator(state, std::marker::PhantomData)
    }
}