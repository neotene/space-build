use std::collections::HashMap;

use crate::world::galaxy::Galaxy;
use crate::world::player::Player;
use crate::world::system::{CenterType, System};
use crate::world::temporal::Temporal;
use crate::{Error, GalaxyCoordsRepr, GalaxyOffsetRepr, Result};
#[cfg(not(feature = "no-crossterm"))]
use crossterm::event::{Event, EventStream, KeyCode};
use futures::stream::{FuturesUnordered, SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use nalgebra::Vector3;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tracing::Level;
use uuid::Uuid;

#[cfg(not(feature = "no-crossterm"))]
async fn crossterm_wrapper_next(prompt: &mut String, crossterm_events: &mut EventStream) {
    let event = crossterm_events.next().await.unwrap().unwrap();
    tracing::trace!("=> On term event");
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char(c) => {
                *prompt = format!("{}{}", prompt, c.to_string());
            }
            KeyCode::Enter => {
                if prompt == "help" {
                    println!("Commands:");
                    println!("\tdatas\t\t(print all datas)");
                } else if prompt == "datas" {
                    // println!("ws_accept_futs size:\t\t{}", ws_accept_futs.len());
                    // println!("first_read_futs size:\t\t{}", first_read_futs.len());
                    // println!("read_futs size:\t\t\t{}", read_futs.len());
                    // println!("writers size:\t\t\t{}", self.writers.len());
                } else {
                    println!("Unkown command {}", prompt);
                }
                *prompt = "".to_string();
            }
            _ => {}
        },
        _ => {}
    }
}

#[cfg(feature = "no-crossterm")]
async fn crossterm_wrapper_next(_: &mut String, _: &mut String) {
    tokio::time::sleep(tokio::time::Duration::from_nanos(1)).await;
}

#[derive(Clone)]
pub enum PlayerAction {
    Login(String),
    Move(Vector3<f32>),
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Login(Login),
    Move(Vector3<f32>),
}

type WsReader = SplitStream<WebSocketStream<TcpStream>>;
type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;

pub struct GameServer {
    galaxy: Galaxy,
    writers: HashMap<Uuid, WsWriter>,
    interrupt_receiver: Receiver<()>,
}

impl GameServer {
    pub fn new(galaxy: Galaxy) -> (Sender<()>, Self) {
        let (interrupt_sender, interrupt_receiver) = mpsc::channel(1);
        (
            interrupt_sender,
            Self {
                galaxy,
                writers: HashMap::new(),
                interrupt_receiver,
            },
        )
    }

    async fn next_message(reader: &mut WsReader) -> Result<ClientMessage> {
        let Some(data) = reader.next().await else {
            tracing::info!("Nothing to read");
            return Err(Error::NothingToRead);
        };

        let msg = data.map_err(|err| Error::WebSocketError(err))?;

        match msg {
            tungstenite::protocol::Message::Text(txt) => {
                let deser_res: serde_json::error::Result<ClientMessage> =
                    serde_json::from_str(&txt);

                match deser_res {
                    Ok(msg) => {
                        return Ok(msg);
                    }
                    Err(err) => Err(Error::ClientMessageDeserializeError(err)),
                }
            }
            tungstenite::Message::Close(_) => Err(Error::NormalClose),
            _ => Err(Error::UnexpectedNonTextMessageError),
        }
    }

    pub async fn read(mut reader: WsReader, uuid: Uuid) -> (WsReader, Uuid, Result<PlayerAction>) {
        match Self::next_message(&mut reader).await {
            Ok(message) => match message {
                _ => (
                    reader,
                    uuid,
                    Ok(PlayerAction::Move(Vector3::new(1., 1., 1.))),
                ),
            },
            Err(err) => (reader, uuid, Err(err)),
        }
    }

    pub async fn first_read(
        writer: WsWriter,
        mut reader: WsReader,
    ) -> (WsWriter, WsReader, Result<PlayerAction>) {
        match Self::next_message(&mut reader).await {
            Ok(message) => match message {
                ClientMessage::Login(login) => {
                    (writer, reader, Ok(PlayerAction::Login(login.nickname)))
                }
                _ => (writer, reader, Err(Error::UnexpectedNonLoginMessage)),
            },
            Err(err) => (writer, reader, Err(err)),
        }
    }

    fn handle_login(&mut self, nickname: String) -> Result<Uuid> {
        tracing::debug!("{nickname} is trying to login");

        match self.galaxy.load_player_by_nickname(nickname.clone()) {
            Ok(uuid) => {
                tracing::info!("Known player '{nickname}' was added to game");
                Ok(uuid)
            }
            Err(Error::NoPlayerForNickname) => {
                let mut rng = rand::thread_rng();
                let x: GalaxyCoordsRepr = rng.gen_range(-15000..15000);
                let y: GalaxyCoordsRepr = rng.gen_range(-2000..2000);
                let z: GalaxyCoordsRepr = rng.gen_range(-15000..15000);

                let offset_x: GalaxyOffsetRepr = rng.gen_range(-100000..100000);
                let offset_y: GalaxyOffsetRepr = rng.gen_range(-100000..100000);
                let offset_z: GalaxyOffsetRepr = rng.gen_range(-100000..100000);

                let player_system = System::new(
                    Vector3::new(x, y, z),
                    Vector3::new(offset_x, offset_y, offset_z),
                    CenterType::from(rng.gen_range(0..4)),
                );

                let player_sys_uuid = self.galaxy.add_system(player_system);
                self.galaxy.save_systems()?;

                let uuid = self.galaxy.add_player(Player::new(
                    Vector3::new(100., 100., 100.),
                    nickname.clone(),
                    player_sys_uuid,
                ));
                self.galaxy.save_players()?;
                tracing::info!("New player '{nickname}' was added to game");
                Ok(uuid)
            }
            Err(Error::NoPlayerForUuid) => {
                tracing::error!("No player for uuid");
                Ok(Uuid::new_v4())
            }
            Err(err) => {
                tracing::error!("Unexpected error when looking for player: {err}");
                Ok(Uuid::new_v4())
            }
        }
    }

    fn player_name(&self, uuid: Uuid) -> String {
        self.galaxy
            .players
            .get(&uuid)
            .map_or("<unknown>".to_string(), |value| value.nickname.clone())
    }
    fn clean_player(&mut self, uuid: Uuid) {
        self.writers.remove(&uuid);
        self.galaxy.players.remove(&uuid);
    }

    pub async fn run(&mut self) -> Result<()> {
        self.galaxy.load_all()?;

        let ref_instant = tokio::time::Instant::now();

        #[cfg(not(feature = "no-crossterm"))]
        let mut crossterm_events = EventStream::new();
        #[cfg(feature = "no-crossterm")]
        let mut crossterm_events = String::new();
        let mut prompt: String = String::new();

        let addr = "127.0.0.1:2567";
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        let mut ws_accept_futs = FuturesUnordered::new();
        let mut first_read_futs = FuturesUnordered::new();
        let mut read_futs = FuturesUnordered::new();

        let mut tick_delay = tokio::time::interval(std::time::Duration::from_millis(250));
        let mut write_delay = tokio::time::interval(std::time::Duration::from_millis(250));

        let subscriber = tracing_subscriber::fmt()
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber).map_err(|_| Error::TracingError)?;

        tracing::trace!("Started");
        loop {
            tokio::select! {
                // ----------------------------------------------------
                // ------------------ON INTERRUPT----------------------
                // ----------------------------------------------------
                interrupt = self.interrupt_receiver.recv() => {
                    if interrupt.is_some() {
                        return Ok(());
                    }
                },
                // ----------------------------------------------------
                // ------------------ON TICK---------------------------
                // ----------------------------------------------------
                _ = tick_delay.tick() => {
                    tracing::trace!("=> On game tick");
                    let now = tokio::time::Instant::now();
                    let delta = now - ref_instant;
                    match self.galaxy.update(delta.as_secs_f32()) {
                        Err(err) => tracing::error!("Galaxy update error: {err}"),
                        Ok(_) => {},
                    }

                    for (uuid, player) in self.galaxy.players.iter_mut() {
                        match self.galaxy.systems.get(&player.current_system_uuid) {
                            None => tracing::error!("Current system for played not found."),
                            Some(system) => {
                                let json_result = serde_json::to_string(system);
                                match json_result {
                                    Err(err) => tracing::error!("Could not serialize a system: {err}"),
                                    Ok(json_str) => {
                                        match self.writers.get_mut(uuid) {
                                            None => tracing::error!("Could not find writer for player {}", player.nickname),
                                            Some(writer) => {
                                                match writer.feed(Message::Text(json_str)).await {
                                                    Err(err) => tracing::warn!("writer feed error: {err}"),
                                                    _ => {}
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                // ----------------------------------------------------
                // ---------------ON TERM EVENT------------------------
                // ----------------------------------------------------
                _ = crossterm_wrapper_next(&mut prompt, &mut crossterm_events) => {},
                // ----------------------------------------------------
                // --------------ON FLUSH TIMER------------------------
                // ----------------------------------------------------
                _ = write_delay.tick() => {
                    tracing::trace!("=> On flush timer");
                    for (_nickname, writer) in self.writers.iter_mut() {
                        match writer.flush().await {
                            Err(err) => tracing::error!("Flush failed: {err}"),
                            Ok(_result) => {},
                        }
                    }
                },
                // ----------------------------------------------------
                // ---------------ON TCP ACCEPT------------------------
                // ----------------------------------------------------
                Ok((stream, _)) = listener.accept() => {
                    tracing::trace!("=> On tcp accept");
                    ws_accept_futs.push(async move {
                        let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");
                        ws_stream.split()
                    });
                },
                // ----------------------------------------------------
                // ---------------ON WS ACCEPT-------------------------
                // ----------------------------------------------------
                Some((writer, reader)) = ws_accept_futs.next() => {
                    tracing::trace!("=> On websocket accept");
                    first_read_futs.push(Self::first_read(writer, reader));
                },
                // ----------------------------------------------------
                // ---------------ON FIRST READ------------------------
                // ----------------------------------------------------
                Some((writer, reader, read_result)) = first_read_futs.next() => {
                    tracing::trace!("=> On first read");
                    match read_result {
                        Err(err) => tracing::warn!("{err}"),
                        Ok(player_action) => match player_action {
                            PlayerAction::Login(nickname) => {
                                match self.handle_login(nickname.clone()) {
                                    Err(err) => tracing::warn!("login failed for {nickname}: {err}"),
                                    Ok(uuid) => {
                                        self.writers.insert(uuid, writer);
                                        read_futs.push(Self::read(reader, uuid));
                                    },
                                }
                            }
                            _ => {}
                        }

                    }
                },
                // ----------------------------------------------------
                // ------------------ON READ---------------------------
                // ----------------------------------------------------
                Some((reader, uuid, read_result)) = read_futs.next() => {
                    tracing::trace!("=> On read");
                    match read_result {
                        Ok(player_action) => match player_action {
                            PlayerAction::Move(_velocity) => {
                                read_futs.push(Self::read(reader, uuid));
                            },
                            _ => {
                                tracing::info!("Unsuported client message, closing");
                                self.clean_player(uuid);
                            },
                        }
                        Err(err) => {
                            match err {
                                Error::NothingToRead => tracing::info!("Nothing to read"),
                                Error::WebSocketError(err) => {
                                    match err {
                                        tungstenite::Error::ConnectionClosed =>
                                            tracing::info!("{}: connection closed", self.player_name(uuid)),
                                        tungstenite::Error::Capacity(err) =>
                                            tracing::error!("{}: capacity error: {err}", self.player_name(uuid)),
                                        tungstenite::Error::Protocol(err) => {
                                            match err {
                                                tungstenite::error::ProtocolError::ResetWithoutClosingHandshake =>
                                                    tracing::info!("{}: connection closed forcefully", self.player_name(uuid)),
                                                _ => tracing::error!("{}: protocol error: {err}", self.player_name(uuid)),
                                            }
                                        }
                                        _ => tracing::error!("{}: unexpected error: {err}", self.player_name(uuid)),
                                    }
                                },
                                Error::NormalClose => {},
                                _ => tracing::info!("{}: Normal close", self.player_name(uuid)),
                            }
                            self.clean_player(uuid);
                        }
                    }
                },
            }
        }
    }
}
