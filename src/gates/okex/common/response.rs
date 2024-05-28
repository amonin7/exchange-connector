use compact_str::CompactString;
use serde::Deserialize;

use crate::gates::okex::common::error::OkexErrorResponse;

#[derive(Debug, Deserialize)]
pub struct OkexResponse<R> {
    code: CompactString,
    data: Vec<R>,
    msg: CompactString,
}

impl<R> OkexResponse<R> {
    pub fn into_result(self) -> Result<Vec<R>, OkexErrorResponse> {
        if self.msg.is_empty() {
            Ok(self.data)
        } else {
            Err(OkexErrorResponse {
                code: self.code,
                msg: self.msg,
            })
        }
    }
}
