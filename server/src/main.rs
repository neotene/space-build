use std::str::FromStr;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize}; // Importer Serialize et Deserialize de serde
                                     // use serde_json::Result;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message; // Importer Result de serde_json

pub enum Block {
    Tile(TileBlock),
    Table(TableBlock),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TileBlock {
    color: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TableBlock {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coords {
    x: usize,
    y: usize,
    z: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockData {
    block_type: String,
    block_coords: Coords,
    block_json: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Data {
    blocks: Vec<BlockData>,
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

            let mut blocks = Vec::new();

            let mut i = 0;

            while i < 100 {
                blocks.push(Block::Tile(TileBlock { color: 0xffffffff }));
                i = i + 1
            }

            let mut data = Data::default();

            i = 0;
            while i < blocks.len() {
                match &blocks[i] {
                    Block::Table(table) => data.blocks.push(BlockData {
                        block_type: String::from_str("table").unwrap(),
                        block_json: serde_json::to_string(&table).unwrap(),
                        block_coords: Coords { x: i, y: 0, z: 0 },
                    }),
                    Block::Tile(tile) => data.blocks.push(BlockData {
                        block_type: String::from_str("tile").unwrap(),
                        block_json: serde_json::to_string(&tile).unwrap(),
                        block_coords: Coords { x: i, y: 0, z: 0 },
                    }),
                }
                i = i + 1;
            }

            let data_json = serde_json::to_string(&data);

            match data_json {
                Ok(serialized) => {
                    println!("Serialized blocks:");
                    println!("{}", serialized);

                    write
                        .send(Message::Text(serialized))
                        .await
                        .expect("Failed to send message");
                }
                Err(err) => {
                    eprintln!("Failed to serialize blocks: {}", err);
                }
            }

            while let Some(Ok(msg)) = read.next().await {
                println!("Received a message from client: {:?}", msg);
            }
        });
    }
}
