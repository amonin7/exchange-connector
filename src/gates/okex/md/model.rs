use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::api::connection::WsMessage;
use crate::model::internal::{L2Increment, L2Snapshot, MdMessage, Side, SingleLot};
use crate::utils::basic_types::{Amount, deserialize_u64, Price};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OkexWsMessage {
    SubEvent(OkexSubEvent<Stream>),
    Combined(OkexWsCombinedMessage),
    Pong,
}

impl WsMessage for OkexWsMessage {
    fn pong() -> Self {
        Self::Pong
    }
}

#[derive(Debug, Deserialize)]
pub struct OkexWsCombinedMessage {
    pub arg: Stream,
    #[serde(rename = "data")]
    pub message: Vec<OkexWsDataMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OkexWsDataMessage {
    BookSnapshot(OkexOrderBookSnapshot),
}

#[derive(Debug, Deserialize)]
pub struct OkexSubEvent<R> {
    pub event: EventType,
    arg: Option<R>,
    code: Option<CompactString>,
    msg: Option<CompactString>,
    #[serde(rename = "connId")]
    conn_id: CompactString,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Stream {
    pub channel: CompactString,
    #[serde(rename = "instId")]
    pub inst_id: CompactString,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Login,
    Subscribe,
    Unsubscribe,
    Error,
    #[serde(rename = "channel-conn-count")]
    ChannelConnCount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkexOrderBookSnapshot {
    /// Order book on sell side
    pub asks: Vec<OkexBookLevel>,
    /// Order book on buy side
    pub bids: Vec<OkexBookLevel>,
    /// Order book generation time, Unix timestamp format in milliseconds, e.g. 1597026383085
    #[serde(deserialize_with = "deserialize_u64")]
    pub ts: u64,
    /// Checksum, implementation details below
    checksum: Option<i64>,
    /// Sequence ID of the last sent message. Only applicable to books, books-l2-tbt, books50-l2-tbt
    pub prev_seq_id: Option<i64>,
    /// Sequence ID of the current message, implementation details below
    pub seq_id: u64,
}

impl OkexOrderBookSnapshot {
    pub fn to_internal_snapshot(&self, symbol: CompactString) -> L2Snapshot {
        let bids = self.bids
            .iter()
            .map(OkexBookLevel::to_single_lot)
            .collect();
        let asks = self.asks
            .iter()
            .map(OkexBookLevel::to_single_lot)
            .collect();
        L2Snapshot {
            exchange_time: Some(self.ts),
            sequence_no: Some(self.seq_id),
            symbol,
            bids,
            asks,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OkexBookLevel {
    pub price: Price,
    pub amount: Amount,
    pub deprecated: CompactString,
    pub orders_number: CompactString,
}

impl OkexBookLevel {
    pub fn to_md(
        &self,
        exchange_time: Option<u64>,
        symbol: CompactString,
        side: Side,
        last_update_id: u64,
        is_eot: bool,
    ) -> MdMessage {
        MdMessage::L2Increment(L2Increment {
            exchange_time,
            sequence_no: Some(last_update_id),
            symbol,
            side,
            price: self.price,
            amount: self.amount,
            is_eot,
        })
    }

    pub fn to_single_lot(&self) -> SingleLot {
        SingleLot {
            price: self.price,
            amount: self.amount,
        }
    }
}

impl Into<SingleLot> for OkexBookLevel {
    fn into(self) -> SingleLot {
        SingleLot {
            price: self.price,
            amount: self.amount,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct WsRequest<R: Serialize> {
    op: EventType,
    args: Vec<R>,
}

impl <R: Serialize> WsRequest<R> {
    pub fn new_subscribe(channels: Vec<R>) -> Self {
        Self {
            op: EventType::Subscribe,
            args: channels,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::gates::okex::md::model::OkexWsMessage;

    #[test]
    fn order_book_parsing() {
        let symbol_str = fs::read_to_string("tests/ws_order_book_update.json").unwrap();
        let symbol: OkexWsMessage = serde_json::from_str(&symbol_str).unwrap();

        assert!(matches!(symbol, OkexWsMessage::Combined { .. }));
    }

    #[test]
    fn order_book_parsing2() {
        let symbol_str = fs::read_to_string("tests/ws_order_book_update2.json").unwrap();
        let symbol: OkexWsMessage = serde_json::from_str(&symbol_str).unwrap();

        assert!(matches!(symbol, OkexWsMessage::Combined { .. }));
    }
}
