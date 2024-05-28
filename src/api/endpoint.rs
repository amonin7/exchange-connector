use http::Method;
use serde::{Deserialize, Serialize};

pub trait Endpoint {
    type Request: Serialize;
    type Response: for<'de> Deserialize<'de>;

    const METHOD: Method;
    const PATH: &'static str;
}
