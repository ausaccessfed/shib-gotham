use std::io;
use std::marker::PhantomData;

use gotham::state::State;
use gotham::handler::HandlerFuture;
use gotham::middleware::{NewMiddleware, Middleware};

trait AttributesTypePhantom<T>: Send + Sync
where
    T: Send
{
}

/// Gotham middleware for receiving Shibboleth attributes and mapping them into a provided type.
pub struct Shibbleware<T> {
    phantom: PhantomData<AttributesTypePhantom<T>>,
}

impl<T> Clone for Shibbleware<T> {
    fn clone(&self) -> Self {
        Shibbleware { phantom: PhantomData }
    }
}

impl<T> NewMiddleware for Shibbleware<T> {
    type Instance = Self;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(self.clone())
    }
}

impl<T> Middleware for Shibbleware<T> {
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        chain(state)
    }
}
