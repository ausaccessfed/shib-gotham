use std::io;
use std::marker::PhantomData;

use gotham::state::State;
use gotham::handler::HandlerFuture;
use gotham::middleware::{NewMiddleware, Middleware};

use authenticated_session::AuthenticatedSession;

trait SessionTypePhantom<T>: Send + Sync
where
    T: Send
{
}

/// Gotham middleware for receiving Shibboleth attributes and mapping them into a provided type.
pub struct Shibbleware<T>
where
    T: AuthenticatedSession,
{
    phantom: PhantomData<SessionTypePhantom<T>>,
}

impl<T> Shibbleware<T>
where
    T: AuthenticatedSession,
{
    pub fn new() -> Shibbleware<T> {
        Shibbleware { phantom: PhantomData }
    }
}

impl<T> Copy for Shibbleware<T>
where
    T: AuthenticatedSession,
{
}

impl<T> Clone for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    fn clone(&self) -> Self {
        Shibbleware { phantom: PhantomData }
    }
}

impl<T> NewMiddleware for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    type Instance = Self;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(*self)
    }
}

impl<T> Middleware for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        chain(state)
    }
}
