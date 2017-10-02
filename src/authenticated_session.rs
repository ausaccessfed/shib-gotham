use serde::{Serialize, Deserialize};

pub trait AuthenticatedSession
    : Default + Serialize + for<'de> Deserialize<'de> + 'static {
    fn is_authenticated(&self) -> bool;
}
