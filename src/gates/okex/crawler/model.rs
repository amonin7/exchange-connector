use compact_str::CompactString;
use serde::Deserialize;

use crate::model::order_book::OrderBook;
use crate::utils::basic_types::{Amount, Price};

#[derive(Debug, Deserialize)]
pub struct OkexOrderBookSnapshot {
    /// Order book on sell side
    asks: Vec<OkexSingleLot>,
    /// Order book on buy side
    bids: Vec<OkexSingleLot>,
    /// Order book generation time
    ts: CompactString
}

impl Into<OrderBook> for OkexOrderBookSnapshot {
    fn into(self) -> OrderBook {
        let bids = self.bids
            .into_iter()
            .map(|l| (l.price, l.amount))
            .collect();
        let asks = self.asks
            .into_iter()
            .map(|l| (l.price, l.amount))
            .collect();
        OrderBook { bids, asks }
    }
}

#[derive(Debug, Deserialize)]
pub struct OkexSingleLot {
    /// depth price
    price: Price,
    /// quantity at the price (number of contracts for derivatives, quantity in base currency for Spot and Spot Margin)
    amount: Amount,
    /// part of a deprecated feature and it is always "0"
    deprecated: CompactString,
    /// the number of orders at the price.
    orders_number: CompactString,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::gates::okex::common::response::OkexResponse;
    use crate::gates::okex::crawler::model::OkexOrderBookSnapshot;

    #[test]
    fn order_book_parsing() {
        let symbol_str = fs::read_to_string("tests/order_book.json").unwrap();
        let response: OkexResponse<OkexOrderBookSnapshot> = serde_json::from_str(&symbol_str).unwrap();
        assert!(response.into_result().is_ok())
    }
}
