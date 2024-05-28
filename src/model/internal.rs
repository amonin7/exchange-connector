use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::utils::basic_types::{Amount, Price};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MdMessage {
    L2Snapshot(L2Snapshot),
    L2Increment(L2Increment),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct L2Snapshot {
    pub exchange_time: Option<u64>,
    pub sequence_no: Option<u64>,
    pub symbol: CompactString,
    pub bids: Vec<SingleLot>,
    pub asks: Vec<SingleLot>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleLot {
    pub price: Price,
    pub amount: Amount,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct L2Increment {
    pub exchange_time: Option<u64>,
    pub sequence_no: Option<u64>,
    pub symbol: CompactString,
    pub side: Side,
    pub price: Price,
    pub amount: Amount,
    pub is_eot: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Side {
    Bid,
    Ask,
}
