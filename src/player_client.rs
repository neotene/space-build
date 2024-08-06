extern crate tokio;
extern crate tokio_tungstenite;

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::{ClientMessage, Error, Login};

use crate::Result;

pub struct PlayerClient {
    writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl PlayerClient {
    pub async fn connect(host: String) -> Result<PlayerClient> {
        let (ws_stream, _) = connect_async(host)
            .await
            .map_err(|err| Error::WebSocketError(err))?;

        let (writer, reader) = ws_stream.split();
        Ok(PlayerClient { writer, reader })
    }

    pub async fn login(&mut self, nickname: String) -> Result<()> {
        let login = ClientMessage::Login(Login { nickname });

        let to_send = serde_json::to_string(&login).unwrap();

        self.writer
            .send(tungstenite::protocol::Message::Text(to_send))
            .await
            .unwrap();

        Ok(())
    }
    pub async fn wait_message(&mut self) -> crate::Result<String> {
        let maybe_read_result = self.reader.next().await;

        if maybe_read_result.is_none() {
            return Err(Error::NothingToRead);
        }

        let read_result = maybe_read_result.unwrap();

        if read_result.is_err() {
            tracing::warn!("Error receiving message: {:?}", read_result.err());
            return Err(Error::NothingToRead);
        }

        let message = read_result.unwrap();

        match message.clone() {
            tungstenite::protocol::Message::Text(text) => {
                return Ok(text);
            }
            tungstenite::protocol::Message::Binary(_bin) => {}
            tungstenite::protocol::Message::Ping(_ping) => {}
            tungstenite::protocol::Message::Pong(_pong) => {}
            tungstenite::protocol::Message::Close(_close) => {}
            tungstenite::protocol::Message::Frame(_frame) => {}
        }
        Err(Error::UnexpectedNonTextMessageError)
    }

    pub async fn run_loop(&mut self) {}
}
