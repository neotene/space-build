extern crate tokio;
extern crate tokio_tungstenite;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite};

use server::{ClientMessage, Login};

#[tokio::main]
async fn main() -> Result<(), String> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|_| "Tracer init error")?;

    let (ws_stream, _) = connect_async("ws://127.0.0.1:2567")
        .await
        .expect("Failed to connect");

    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    let login = ClientMessage::Login(Login {
        nickname: "killer".to_string(),
    });

    let to_send = serde_json::to_string(&login).map_err(|_| "Serialization error".to_string())?;

    write
        .send(tungstenite::protocol::Message::Text(to_send))
        .await
        .unwrap();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => match msg {
                tungstenite::protocol::Message::Text(text) => {
                    println!("Received text message: {}", text);
                }
                tungstenite::protocol::Message::Binary(bin) => {
                    println!("Received binary message: {:?}", bin);
                }
                tungstenite::protocol::Message::Ping(ping) => {
                    println!("Received ping: {:?}", ping);
                }
                tungstenite::protocol::Message::Pong(pong) => {
                    println!("Received pong: {:?}", pong);
                }
                tungstenite::protocol::Message::Close(close) => {
                    println!("Received close message: {:?}", close);
                    break;
                }
                tungstenite::protocol::Message::Frame(_frame) => {
                    println!("Received frame message");
                }
            },
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    println!("WebSocket connection closed");

    Ok(())
}
