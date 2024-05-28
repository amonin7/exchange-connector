use compact_str::{CompactString, ToCompactString};

use crate::gates::okex::md::model::{Stream, WsRequest};
use crate::model::stream::WsStream;

#[derive(Clone)]
pub struct OkexStream {
    pub tickers: Vec<CompactString>,
    pub kind: OkexStreamKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OkexStreamKind {
    /// books: 400 depth levels will be pushed in the initial full snapshot.
    /// Incremental data will be pushed every 100 ms for the changes in the order book during that period of time.
    ///
    /// https://www.okx.com/docs-v5/en/#order-book-trading-market-data-ws-order-book-channel
    L2Update,
}

impl WsStream for OkexStream {
    type Kind = OkexStreamKind;
    type Subscribe = WsRequest<Stream>;

    fn kind(&self) -> Self::Kind {
        self.kind
    }

    fn subscribe_requests(&self) -> Vec<Self::Subscribe> {
        let channel = match self.kind {
            OkexStreamKind::L2Update => "books",
        }.to_compact_string();

        let requests = self.tickers
            .iter()
            .map(|inst_id| Stream { channel: channel.clone(), inst_id: inst_id.clone() })
            .collect();

        vec![WsRequest::new_subscribe(requests)]
    }
}