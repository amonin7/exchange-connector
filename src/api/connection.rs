use async_trait::async_trait;

use crate::model::internal::MdMessage;

#[async_trait]
pub trait MdConnection: Send {
    async fn next(&mut self) -> eyre::Result<MdMessage>;
}

pub trait WsMessage {
    fn pong() -> Self;
}
