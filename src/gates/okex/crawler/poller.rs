use async_trait::async_trait;
use compact_str::CompactString;
use eyre::OptionExt;

use crate::api::api;
use crate::api::poller::ExchangePoller;
use crate::gates::okex::crawler::config::OkexPollerConfig;
use crate::gates::okex::crawler::endpoints::GetOrderBook;
use crate::gates::okex::crawler::request::GetOrderBookRequest;
use crate::model::order_book::OrderBook;

#[derive(Debug, Default)]
pub struct OkexExchangePoller {
    pub config: OkexPollerConfig,
}

#[async_trait]
impl ExchangePoller for OkexExchangePoller {
    async fn get_order_book(&self, symbol: CompactString) -> eyre::Result<OrderBook> {
        let request = GetOrderBookRequest::new(symbol, None);

        let response = api::http_urlencoded_query_request::<GetOrderBook>(
            &self.config.http_url,
            &request,
            Default::default(),
        ).await?;

        let info = response.into_result()?;

        let ob = info
            .into_iter()
            .next()
            .ok_or_eyre("There was no order book returned from Okex API")?
            .into();

        Ok(ob)
    }
}

impl OkexExchangePoller {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_config(config: OkexPollerConfig) -> Self {
        Self { config }
    }
}

#[cfg(test)]
mod api_tests {
    use crate::api::poller::ExchangePoller;
    use crate::gates::okex::crawler::poller::OkexExchangePoller;

    #[tokio::test]
    async fn get_ob_test() {
        let poller = OkexExchangePoller::new();
        let ob = poller.get_order_book("BTC-USDT".into()).await;
        assert!(ob.is_ok());
        let ob = ob.unwrap();
        assert_ne!(ob.bids.len(), 0);
        assert_ne!(ob.asks.len(), 0);
    }

    #[tokio::test]
    async fn get_ob_test_instrument_not_exist() {
        let poller = OkexExchangePoller::new();
        let ob = poller.get_order_book("BTCUSDT".into()).await;
        assert!(ob.is_err());
    }
}