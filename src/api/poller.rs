use async_trait::async_trait;
use compact_str::CompactString;

use crate::model::order_book::OrderBook;

#[async_trait]
pub trait ExchangePoller {
    async fn get_order_book(&self, symbol: CompactString) -> eyre::Result<OrderBook>;
}