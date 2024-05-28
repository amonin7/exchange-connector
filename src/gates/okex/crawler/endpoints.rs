use http::Method;

use crate::api::endpoint::Endpoint;
use crate::gates::okex::common::response::OkexResponse;
use crate::gates::okex::crawler::model::OkexOrderBookSnapshot;
use crate::gates::okex::crawler::request::GetOrderBookRequest;

pub struct GetOrderBook;

impl Endpoint for GetOrderBook {
    type Request = GetOrderBookRequest;
    type Response = OkexResponse<OkexOrderBookSnapshot>;

    const METHOD: Method = Method::GET;
    const PATH: &'static str = "/api/v5/market/books";
}
