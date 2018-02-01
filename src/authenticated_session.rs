use serde::{Deserialize, Serialize};

pub trait AuthenticatedSession
    : Default + Serialize + for<'de> Deserialize<'de> + 'static {
    fn is_authenticated(&self) -> bool;
}
