use compact_str::CompactString;
use derive_more::Display;
use serde::Deserialize;

#[derive(Debug, Display, Deserialize)]
#[display("{code}:{msg}")]
pub struct OkexErrorResponse {
    pub code: CompactString,
    pub msg: CompactString,
}

impl std::error::Error for OkexErrorResponse {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}
