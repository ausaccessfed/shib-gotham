pub trait AuthenticatedSession {
    fn is_authenticated(&self) -> bool;
}
