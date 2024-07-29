use super::{player::Player, system::System, temporal::Temporal};
use crate::error::Error;
use crate::Result;
use redis::{Commands, RedisResult};
use regex::Regex;
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

pub struct Galaxy {
    pub connection: redis::Connection,
    pub systems: HashMap<Uuid, System>,
    pub players: HashMap<Uuid, Player>,
    pub db_name: String,
    pub rotation_speed: f32,
}

impl Galaxy {
    pub fn new(db_name: String) -> Result<Self> {
        let client =
            redis::Client::open("redis://127.0.0.1/").map_err(|_| Error::RedisOpenError)?;
        let connection = client
            .get_connection()
            .map_err(|err| Error::RedisGetConnError(err))?;

        Ok(Self {
            connection,
            systems: HashMap::new(),
            players: HashMap::new(),
            db_name,
            rotation_speed: 1.,
        })
    }

    pub fn add_system(&mut self, system: System) -> Uuid {
        let uuid = Uuid::new_v4();
        self.systems.insert(uuid, system);
        uuid
    }

    pub fn save_systems(&mut self) -> Result<()> {
        let mut jsons: Vec<(Uuid, String)> = Vec::new();

        for (uuid, system) in &self.systems {
            jsons.push((
                *uuid,
                serde_json::to_string(system)
                    .map_err(|err| Error::SaveSystemsSerializationError(err))?,
            ));
        }

        for (uuid, json) in jsons {
            self.connection
                .set(format!("{}:system:{}", self.db_name, uuid), json)
                .map_err(|err| Error::SaveSystemsSetError(err))?;
        }
        Ok(())
    }

    pub fn load_systems(&mut self) -> Result<()> {
        let keys: Vec<String> = self
            .connection
            .keys(format!("{}:system:*", self.db_name))
            .map_err(|_| Error::KeysQueryError)?;

        let re = Regex::new(r"^.*system:(.*)$").unwrap();

        let systems: Vec<(Uuid, System)> = keys
            .iter()
            .filter_map(|system_key: &String| {
                let mut results = vec![];
                for (_, [uuid]) in re.captures_iter(system_key).map(|c| c.extract()) {
                    results.push(uuid);
                }

                assert!(results.len() == 1);

                let uuid = match Uuid::from_str(results[0]) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        return None;
                    }
                };
                let res: RedisResult<String> = self.connection.get(system_key);
                match res {
                    Ok(val) => {
                        let res: serde_json::Result<System> = serde_json::from_str(&val);
                        match res {
                            Ok(system) => Some((uuid, system)),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect();

        for (uuid, system) in systems {
            self.systems.insert(uuid, system);
        }

        Ok(())
    }

    pub fn add_player(&mut self, player: Player) -> Uuid {
        let uuid = Uuid::new_v4();
        self.players.insert(uuid, player);
        uuid
    }

    pub fn save_players(&mut self) -> Result<()> {
        let jsons: Vec<(Uuid, String, String)> = self
            .players
            .iter()
            .filter_map(
                |(key, value): (&Uuid, &Player)| match serde_json::to_string(&value) {
                    Ok(json) => Some((key.clone(), json, value.nickname.clone())),
                    Err(_) => {
                        tracing::warn!("json error");
                        None
                    }
                },
            )
            .collect();

        jsons.iter().for_each(|(uuid, json, nickname)| {
            let key_name = format!("{}:player:{}", self.db_name, uuid);
            let _: RedisResult<()> = self.connection.set(key_name.clone(), json);

            let _: RedisResult<()> = self.connection.set(
                format!("{}:nickname_to_uuid:{}", self.db_name, nickname),
                uuid.to_string(),
            );
        });

        Ok(())
    }

    pub fn load_player_by_nickname(&mut self, nickname: String) -> Result<Uuid> {
        let player_uuid: String = self
            .connection
            .get(format!("{}:nickname_to_uuid:{}", self.db_name, nickname))
            .map_err(|_| Error::NoPlayerForNickname)?;

        let json: String = self
            .connection
            .get(format!("{}:player:{}", self.db_name, player_uuid))
            .map_err(|_| Error::NoPlayerForUuid)?;

        let player =
            serde_json::from_str(&json).map_err(|err| Error::PlayerDeserializationError(err))?;

        let uuid: Uuid = Uuid::from_str(player_uuid.as_str()).map_err(|_| Error::UuidError)?;

        self.players.insert(uuid, player);
        Ok(uuid)
    }

    pub fn load_all(&mut self) -> Result<()> {
        self.load_systems()?;
        Ok(())
    }

    pub fn save_all(&mut self) -> Result<()> {
        self.save_systems()?;
        Ok(())
    }

    pub fn clear_db(&mut self) -> Result<()> {
        let all_keys = self.all_keys()?;

        for key in all_keys {
            self.connection
                .del(key)
                .map_err(|_| Error::DeletionQueryError)?;
        }
        Ok(())
    }

    fn all_keys(&mut self) -> Result<Vec<String>> {
        Ok(self
            .connection
            .keys(format!("{}:*", self.db_name))
            .map_err(|_| Error::KeysQueryError)?)
    }
}

impl Temporal for Galaxy {
    fn update(&mut self, delta: f32) -> Result<()> {
        for (_uuid, system) in self.systems.iter_mut() {
            system.update(delta)?;
        }

        Ok(())
    }
}
