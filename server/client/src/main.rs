extern crate tokio;
extern crate tokio_tungstenite;

use futures_util::{SinkExt, StreamExt};
// use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let url = url::Url::parse("ws://localhost:2567").unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // On peut envoyer un message initial au serveur si nécessaire
    write
        .send(Message::Text("Hello WebSocket".into()))
        .await
        .unwrap();

    // Boucle pour lire et afficher les messages du serveur
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    println!("Received text message: {}", text);
                }
                Message::Binary(bin) => {
                    println!("Received binary message: {:?}", bin);
                }
                Message::Ping(ping) => {
                    println!("Received ping: {:?}", ping);
                }
                Message::Pong(pong) => {
                    println!("Received pong: {:?}", pong);
                }
                Message::Close(close) => {
                    println!("Received close message: {:?}", close);
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    println!("WebSocket connection closed");
}
