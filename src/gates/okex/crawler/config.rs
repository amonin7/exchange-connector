use compact_str::CompactString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OkexPollerConfig {
    pub http_url: CompactString,
}

impl Default for OkexPollerConfig {
    fn default() -> Self {
        Self {
            http_url: "https://www.okx.com".into(),
        }
    }
}
