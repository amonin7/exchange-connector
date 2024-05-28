use std::collections::VecDeque;
use std::time::Duration;

use async_trait::async_trait;
use compact_str::ToCompactString;
use eyre::{eyre, Result};
use log::{error, trace, warn};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::api::connection::MdConnection;
use crate::api::ws::WebSocket;
use crate::gates::okex::md::config::OkexMdConnectionConfig;
use crate::gates::okex::md::model::{EventType, OkexWsDataMessage, OkexWsMessage};
use crate::gates::okex::md::stream::{OkexStream, OkexStreamKind};
use crate::model::internal::{MdMessage, Side};

pub struct OkexMdConnection {
    ws: WebSocket<OkexStream, OkexWsMessage>,
    increment_queue: VecDeque<MdMessage>,
    rx: Receiver<()>,
}

impl OkexMdConnection {
    /// If you need to subscribe to many 50 or 400 depth level channels,
    /// it is recommended to subscribe through multiple websocket connections, with each of less than 30 channels.
    /// https://www.okx.com/docs-v5/en/#order-book-trading-market-data-ws-order-book-channel
    pub async fn new(tickers: Vec<impl ToCompactString>, config: OkexMdConnectionConfig) -> Self {
        let tickers: Vec<_> = tickers.into_iter().map(|s| s.to_compact_string()).collect();
        let stream = OkexStream { tickers, kind: OkexStreamKind::L2Update, };
        let mut ws = WebSocket::try_establish_connection(
            &config.ws_url,
            stream,
            config.subscribe_interval_ms
        ).await
            .expect("Failed to connect to Okex websocket");
        ws.subscribe().await
            .expect("Failed to subscribe to Okex md");

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(Self::ping_task(tx, config.ping_frequency_seconds));

        Self {
            ws,
            increment_queue: VecDeque::new(),
            rx,
        }
    }

    async fn ping_task(tx: Sender<()>, frequency: u64) {
        let mut interval = tokio::time::interval(Duration::from_secs(frequency));
        interval.tick().await;
        loop {
            interval.tick().await;
            let res = tx.send(()).await;
            if res.is_err() {
                error!("Failed to send Ping to Okex channel");
            }
        }
    }
}

#[async_trait]
impl MdConnection for OkexMdConnection {
    async fn next(&mut self) -> Result<MdMessage> {
        // If there’s a network problem, the system will automatically disable the connection.
        // The connection will break automatically if the subscription is not established or
        // data has not been pushed for more than 30 seconds.
        // To keep the connection stable:
        // 1. Set a timer of N seconds whenever a response message is received, where N is less than 30.
        // 2. If the timer is triggered, which means that no new message is received within N seconds,
        // send the String 'ping'.
        // 3. Expect a 'pong' as a response. If the response message is not received within N seconds,
        // please raise an error or reconnect.
        //
        // https://www.okx.com/docs-v5/en/#overview-websocket-connect

        while self.increment_queue.is_empty() {
            tokio::select! {
                res = self.ws.next() => {
                    let ws_message = res?;

                    match ws_message {
                        OkexWsMessage::Combined(combined) => {
                            if combined.message.len() != 1 {
                                return Err(eyre!("Failed to deserialize: The incoming message length does not equal 1. {combined:?}"));
                            }
                            return match combined.message.get(0).unwrap() {
                                OkexWsDataMessage::BookSnapshot(snapshot) => {
                                    // maybe this logic should be extracted to model.rs
                                    let instrument_id = combined.arg.inst_id;
                                    if let Some(prev_seq_id) = snapshot.prev_seq_id {
                                        if prev_seq_id != -1 {
                                            let bids = snapshot.bids
                                                .iter()
                                                .enumerate()
                                                .map(|(i, b)| b.to_md(
                                                    Some(snapshot.ts),
                                                    instrument_id.clone(),
                                                    Side::Bid,
                                                    snapshot.seq_id,
                                                    i + 1 == snapshot.bids.len() && snapshot.asks.len() == 0,
                                                ));
                                            let asks = snapshot.asks
                                                .iter()
                                                .enumerate()
                                                .map(|(i, a)| a.to_md(
                                                    Some(snapshot.ts),
                                                    instrument_id.clone(),
                                                    Side::Ask,
                                                    snapshot.seq_id,
                                                    i + 1 == snapshot.asks.len()
                                                ));
                                            self.increment_queue.extend(bids);
                                            self.increment_queue.extend(asks);
                                            break;
                                        }
                                    }
                                    Ok(MdMessage::L2Snapshot(snapshot.to_internal_snapshot(instrument_id)))
                                }
                            }
                        },
                        OkexWsMessage::SubEvent(sub) => {
                            if matches!(sub.event, EventType::Error) {
                                warn!("received error subscribe event {sub:?}")
                            } else {
                                trace!("sub event");
                            }
                            continue;
                        }
                        OkexWsMessage::Pong => {
                            continue;
                        }
                    }
                }
                ping = self.rx.recv() => {
                    if let Some(_ping_msg) = ping {
                        let res = self.ws.ping(vec![]).await;
                        if res.is_err() {
                            error!("Failed to send Ping to Okex MD stream");
                        }
                    }
                }
            }
        }
        let update = self.increment_queue.pop_front().expect("should be some");
        Ok(update)
    }
}

#[cfg(test)]
mod md_integration_tests {
    use std::time::Duration;

    use crate::api::connection::MdConnection;
    use crate::gates::okex::md::config::OkexMdConnectionConfig;
    use crate::gates::okex::md::connection::OkexMdConnection;
    use crate::model::storage::Storage;

    #[ignore]
    #[tokio::test]
    async fn ob_test() {
        log4rs::init_file("logging.yaml", Default::default()).expect("logger initialisation failure");

        let mut storage = Storage::new();

        let mut connection = OkexMdConnection::new(
            vec!["BTC-USDT", "ETH-USDT"],
            OkexMdConnectionConfig::default(),
        ).await;

        loop {
            let m = connection.next().await.unwrap();
            storage.on_ws_update(m);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
