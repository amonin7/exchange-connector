use compact_str::CompactString;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookRequest {
    /// Instrument ID, e.g. BTC-USDT
    inst_id: CompactString,
    /// Order book depth per side. Maximum 400, e.g. 400 bids + 400 asks
    /// Default returns to 1 depth data
    sz: Option<u16>,
}

impl GetOrderBookRequest {
    pub fn new(symbol: CompactString, limit: Option<u16>) -> Self {
        Self {
            inst_id: symbol,
            sz: limit,
        }
    }
}