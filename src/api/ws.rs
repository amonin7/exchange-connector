use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;

use compact_str::CompactString;
use eyre::bail;
use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use http::StatusCode;
use log::{error, trace, warn};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::handshake::client::Response;

use crate::api::connection::WsMessage;
use crate::model::stream::WsStream;

pub type WebSocketStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct WebSocket<S, M>
    where
        S: WsStream + Send + 'static + Clone,
{
    ws_stream: WebSocketStream,
    stream: S,
    pub ws_url: CompactString,
    subscribe_interval_ms: u64,
    _phantom_m: PhantomData<M>,
}

impl<S, M> WebSocket<S, M>
    where
        S: WsStream + Send + 'static + Clone,
        <S as WsStream>::Subscribe: Debug,
        M: for<'de> Deserialize<'de> + WsMessage,
{

    pub async fn try_establish_connection(url: &CompactString, stream: S, subscribe_interval_ms: u64) -> eyre::Result<Self> {
        let (ws_stream, _response) = Self::connect(url).await;
        Ok(Self {
            ws_stream,
            stream,
            ws_url: url.clone(),
            subscribe_interval_ms,
            _phantom_m: Default::default(),
        })
    }

    async fn connect(url: &CompactString) -> (WebSocketStream, Response) {
        let (stream, response) = connect_async(url.as_str())
            .await
            .expect("WebSocket connection failed");
        trace!("WebSocket connection established, response = {response:?}");
        (stream, response)
    }
    
    pub async fn reconnect(&mut self) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if let Err(err) = self.ws_stream.close(None).await {
            warn!("websocket stream close failure while reconnecting, {err}");
        }
        trace!("Trying to reconnect to {}", self.ws_url);
        let (ws_stream, _response) = Self::connect(&self.ws_url).await;
        trace!("Reconnected to {}", self.ws_url);
        self.ws_stream = ws_stream;
        self.subscribe().await.expect("subscribe failure while reconnecting");
        trace!("Resubscribed to url {}", &self.ws_url);
    }

    pub async fn subscribe(&mut self) -> eyre::Result<()> {
        for subscribe_request in self.stream.subscribe_requests() {
            let string = serde_json::to_string(&subscribe_request)?;
            self.ws_stream.send(Message::text(string)).await?;
            if self.subscribe_interval_ms != 0 {
                tokio::time::sleep(Duration::from_millis(self.subscribe_interval_ms)).await;
            }
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Message, Error> {
        let Some(result) = self.ws_stream.next().await else {
            warn!(
                    "IO error: An existing connection was forcibly closed by the remote host.\
                 (os error 10054) - next() returned None; stream url = {:?}; reconnecting...", self.ws_url
                );
            return Ok(Message::Close(None))
        };

        match result {
            Ok(r) => Ok(r),
            Err(err) => match err {
                Error::Protocol(ProtocolError::ResetWithoutClosingHandshake) => {
                    warn!("{}; reconnecting to {}", err, self.ws_url);
                    return Ok(Message::Close(None))
                }
                Error::Http(response) if response.status() == StatusCode::TOO_MANY_REQUESTS => {
                    error!("HTTP status: TOO_MANY_REQUESTS => 1min sleep before reconnecting to {}",  self.ws_url);
                    return Ok(Message::Close(None))
                }
                _ => Err(err),
            },
        }
    }

    pub async fn next(&mut self) -> eyre::Result<M> {
        loop {
            let message = self.recv().await?;
            return match message {
                // .map_err(|_| eyre!("Failed to deserialize message: {s}"))
                Message::Text(s) => Ok(serde_json::from_str::<M>(&s)?),
                Message::Binary(data) => Ok(serde_json::from_slice::<M>(&data)?),
                Message::Close(_) => {
                    self.reconnect().await;
                    continue;
                },
                Message::Pong(_) => Ok(M::pong()),
                _ => bail!("unsupported websocket message"),
            };
        }

    }

    pub async fn ping(&mut self, data: Vec<u8>) -> eyre::Result<()> {
        self.ws_stream.send(Message::Ping(data.clone())).await?;
        Ok(())
    }
}
