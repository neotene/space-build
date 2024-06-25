use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};

use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use rusqlite::params;
use rusqlite::Connection;
use std::result::Result;

use tokio::sync::mpsc;

use nalgebra::Vector3;

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

pub struct ChunkRaw {
    x: i32,
    y: i32,
    blocks: Vec<u16>,
}

#[derive(Debug)]
pub enum Error {
    InitDbError,
    ChunkLoadError,
}

pub struct WorldDB {
    connection: rusqlite::Connection,
}

impl WorldDB {
    fn new(db_path: String) -> Result<Self, Error> {
        if !Path::new(&db_path).exists() {
            File::create(&db_path).map_err(|_| Error::InitDbError)?;
        }

        let connection = Connection::open(&db_path).map_err(|_| Error::InitDbError)?;

        {
            let mut stmt = connection
                .prepare(
                    "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                data BLOB NOT NULL
            )",
                )
                .map_err(|_| Error::InitDbError)?;
            stmt.execute([]).map_err(|_| Error::InitDbError)?;
        }

        Ok(Self { connection })
    }

    fn load_chunk(conn: &Connection, x: i32, y: i32) -> Result<ChunkRaw, Error> {
        let mut stmt = conn
            .prepare("SELECT id, x, y, data FROM chunks WHERE x = ?1 AND y = ?2")
            .map_err(|_| Error::ChunkLoadError)?;
        let mut rows = stmt
            .query(params![x, y])
            .map_err(|_| Error::ChunkLoadError)?;

        if let Some(row) = rows.next().map_err(|_| Error::ChunkLoadError)? {
            let data: Vec<u8> = row.get(3).map_err(|_| Error::ChunkLoadError)?;
            let data: Vec<u16> = bincode::deserialize(&data).map_err(|_| Error::ChunkLoadError)?;
            Ok(ChunkRaw {
                // id: row.get(0)?,
                x: row.get(1).map_err(|_| Error::ChunkLoadError)?,
                y: row.get(2).map_err(|_| Error::ChunkLoadError)?,
                blocks: data,
            })
        } else {
            Err(Error::ChunkLoadError)
        }
    }
}

async fn start(world_db: &mut WorldDB) {}

enum GameInfo {
    Tick(EntitiesAroundPlayer),
}

enum Entity {
    Shop(ShopEntity),
    OtherPlayer(OtherPlayerEntity),
    OtherShip(OtherShipEntity),
    NPC(NPCEntity),
    Asteroid(AsteroidEntity),
}

struct ShopEntity {
    height: u8,
}

struct OtherPlayerEntity {
    vector_to_player: Vector3<f32>,
    gender: bool,
}

struct OtherShipEntity {
    vector_to_ship: Vector3<f32>,
    kind: u16,
}

struct NPCEntity {
    vector_to_npc: Vector3<f32>,
    kind: u16,
}

struct AsteroidEntity {
    vector_to_asteroid: Vector3<f32>,
    kind: u8,
}

struct EntitiesAroundPlayer {
    entities: Vec<Entity>,
}

enum PlayerAction {
    Move(MoveAction),
}

struct MoveAction {
    up: bool,
    down: bool,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

struct EventLoop {
    player_action_sender: mpsc::UnboundedSender<GameInfo>,
    game_info_receiver: mpsc::UnboundedReceiver<GameInfo>,
}

pub struct SpaceBuildServer {
    world_db: WorldDB,
    player_action_receiver: mpsc::UnboundedReceiver<GameInfo>,
    player_action_sender: mpsc::UnboundedSender<GameInfo>,
    game_info_receiver: mpsc::UnboundedReceiver<GameInfo>,
    game_info_sender: mpsc::UnboundedSender<GameInfo>,
}

impl SpaceBuildServer {
    pub fn new(db_path: String) -> Result<Self, Error> {
        let world_db = WorldDB::new(db_path)?;
        let (player_action_sender, player_action_receiver) = mpsc::unbounded_channel();
        let (game_info_sender, game_info_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            world_db,
            player_action_receiver,
            player_action_sender,
            game_info_receiver,
            game_info_sender,
        })
    }

    pub async fn start_game_loop() -> Result<(), Error> {
        Ok(())
    }

    pub async fn start_network_loop(&self) -> Result<(), Error> {
        let addr = "127.0.0.1:2567";
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

        println!("Listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let ws_stream = accept_async(stream)
                    .await
                    .expect("Error during the websocket handshake");

                println!("New connection");

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

                        // write.send(Message::Binary(()))
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
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let game = SpaceBuildServer::new(String::from_str("world.db").unwrap())?;

    tokio::try_join!(game.start_network_loop(), game.start_game_loop())?;
    Ok(())
}
