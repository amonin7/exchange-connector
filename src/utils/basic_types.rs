use std::str::FromStr;

use compact_str::CompactString;
use fixnum::FixedPoint;
use fixnum::typenum::U16;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;

pub type Amount = FixedPoint<i128, U16>;
pub type Price = FixedPoint<i128, U16>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct InstrumentId(u64);

impl Into<InstrumentId> for u64 {
    fn into(self) -> InstrumentId {
        InstrumentId(self)
    }
}

pub fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = CompactString::deserialize(deserializer)?;
    u64::from_str(s.as_str()).map_err(|_| D::Error::custom(format!("non-integer {s}")))
}
