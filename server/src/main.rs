use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use serde::{Serialize, Deserialize}; // Importer Serialize et Deserialize de serde
use serde_json::Result; // Importer Result de serde_json

#[derive(Debug, Serialize, Deserialize)]
pub enum Block
{
    Tile(TileBlock),
    Table(TableBlock)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileBlock
{
    color: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableBlock
{
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data
{
    blocks: Vec<Block>
}

#[tokio::main]
async fn main() {
    // Bind the TCP listener to the specified address
    let addr = "127.0.0.1:2567";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        // Spawn a new task to handle the WebSocket connection
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake");

            println!("New WebSocket connection");

            let (mut write, mut read) = ws_stream.split();

            // let mut my_vec: Vec<u8> = Vec::new();
            // my_vec.push(42);

            let mut data = Data::new();

            let tile1 = TileBlock {
                color: 0xffffffff
            };

            data.blocks.push(Block::Tile(tile1));
            

            let result = serde_json::to_string_pretty(&data);

            match result {
                Ok(serialized) => {
                    println!("Serialized blocks:");
                    println!("{}", serialized);

                    write.send(Message::Text(serialized)).await.expect("Failed to send message");
                }
                Err(err) => {
                    eprintln!("Failed to serialize blocks: {}", err);
                    // Autres actions en cas d'erreur, comme retourner une erreur ou quitter
                    // std::process::exit(1); // Exemple : quitter le programme avec un code d'erreur
                }
            }

            // write.send(Message::Binary(blocks)).await.expect("Failed to send map");


            // write.send(Message::Text("hello".to_string()))
            //     .await
            //     .expect("Failed to send message");

            // Optional: Handle incoming messages from the client
            while let Some(Ok(msg)) = read.next().await {
                println!("Received a message from client: {:?}", msg);
            }
        });
    }
}
