use compact_str::CompactString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OkexMdConnectionConfig {
    pub channel_tickers_amount: usize,
    pub ws_url: CompactString,
    pub ping_frequency_seconds: u64,
    pub subscribe_interval_ms: u64,
}

impl Default for OkexMdConnectionConfig {
    fn default() -> Self {
        Self {
            channel_tickers_amount: 60,
            ws_url: "wss://ws.okx.com:8443/ws/v5/public".into(),
            ping_frequency_seconds: 3,
            subscribe_interval_ms: 100,
        }
    }
}
