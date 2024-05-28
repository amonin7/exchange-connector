use std::hash::Hash;

use serde::Serialize;

pub trait WsStream {
    type Kind: Eq + Hash;
    type Subscribe: Serialize + Send + 'static + Clone;

    fn kind(&self) -> Self::Kind;

    fn subscribe_requests(&self) -> Vec<Self::Subscribe>;
}
