pub mod error;
pub mod game_server;
pub mod player_client;
pub mod world;

pub use error::Error;
pub use game_server::ClientMessage;
pub use game_server::GameServer;
pub use game_server::Login;
pub use world::galaxy::Galaxy;
pub type Result<T> = std::result::Result<T, Error>;
pub type GalaxyCoordsRepr = i16; // parsec
pub type GalaxyOffsetRepr = i32; // au
pub type SystemCoordsRepr = f32; // meter

#[cfg(test)]
mod tests_galaxy {
    use nalgebra::Vector3;
    use redis::Commands;
    use serial_test::serial;
    use std::str::FromStr;
    use uuid::Uuid;
    use world::{
        body::{Body, BodyType},
        player::Player,
        system::{CenterType, System},
    };

    use super::*;

    fn get_test_system_1() -> System {
        System {
            coords: Vector3::new(0, 0, 1),
            offset: Vector3::new(0, 0, 2),
            center_type: CenterType::NeutronStar,
            bodies: vec![
                Body {
                    coords: Vector3::new(0., 0., 3.),
                    velocity: Vector3::new(0., 0., 4.),
                    body_type: BodyType::Planet,
                },
                Body {
                    coords: Vector3::new(0., 0., 5.),
                    velocity: Vector3::new(0., 0., 6.),
                    body_type: BodyType::Station,
                },
            ],
        }
    }

    fn get_test_player_1() -> Player {
        Player {
            coords: Vector3::new(0., 1., 1.),
            current_system_uuid: Uuid::from_str("f599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
            nickname: "test_nick1".to_string(),
            own_system_uuid: Uuid::from_str("f599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
        }
    }

    fn get_test_player_2() -> Player {
        Player {
            coords: Vector3::new(0., 1., 2.),
            current_system_uuid: Uuid::from_str("e599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
            nickname: "test_nick2".to_string(),
            own_system_uuid: Uuid::from_str("e599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
        }
    }

    #[test]
    #[serial]
    fn test_01_clear_db() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests")?;

        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut conn = client.get_connection()?;

        let key1 = "space_build_tests:test1";
        let key2 = "space_build_tests:test2:test3";
        let key3 = "space_build_tests:test4";
        let val = "non";
        let _ = conn.set(key1, val)?;
        let _ = conn.set(key2, val)?;
        let _ = conn.set(key3, val)?;

        galaxy.clear_db()?;

        let keys: Vec<String> = conn.keys("space_build_test:*")?;

        assert_eq!(0, keys.len());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_02_add_system() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests")?;
        galaxy.clear_db()?;
        let uuid = galaxy.add_system(get_test_system_1());
        assert!(galaxy.systems.contains_key(&uuid));
        let system_ref = galaxy.systems.get(&uuid).unwrap();

        assert_eq!(Vector3::new(0, 0, 1), system_ref.coords);
        assert_eq!(Vector3::new(0, 0, 2), system_ref.offset);
        assert_eq!(CenterType::NeutronStar, system_ref.center_type);

        assert_eq!(2, system_ref.bodies.len());
        let body1 = system_ref.bodies.get(0).unwrap();
        let body2 = system_ref.bodies.get(1).unwrap();

        assert_eq!(Vector3::new(0., 0., 3.), body1.coords);
        assert_eq!(Vector3::new(0., 0., 4.), body1.velocity);
        assert_eq!(BodyType::Planet, body1.body_type);

        assert_eq!(Vector3::new(0., 0., 5.), body2.coords);
        assert_eq!(Vector3::new(0., 0., 6.), body2.velocity);
        assert_eq!(BodyType::Station, body2.body_type);
        Ok(())
    }

    #[test]
    #[serial]
    fn test_03_save_systems() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests")?;
        galaxy.clear_db()?;
        let system = get_test_system_1();
        let uuid = galaxy.add_system(system);

        galaxy.save_systems()?;

        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut conn = client.get_connection()?;

        let system_from_redis_json: String =
            conn.get(format!("space_build_tests:system:{uuid}"))?;

        let system_from_redis = serde_json::from_str::<System>(&system_from_redis_json)?;
        let body1_from_redis = system_from_redis.bodies.get(0).unwrap();
        let body2_from_redis = system_from_redis.bodies.get(1).unwrap();

        let system_ref = galaxy.systems.get(&uuid).unwrap();
        let body1 = system_ref.bodies.get(0).unwrap();
        let body2 = system_ref.bodies.get(1).unwrap();

        assert_eq!(system_ref.coords, system_from_redis.coords);
        assert_eq!(system_ref.offset, system_from_redis.offset);
        assert_eq!(system_ref.center_type, system_from_redis.center_type);

        assert_eq!(body1.coords, body1_from_redis.coords);
        assert_eq!(body1.velocity, body1_from_redis.velocity);
        assert_eq!(body1.body_type, body1_from_redis.body_type);

        assert_eq!(body2.coords, body2_from_redis.coords);
        assert_eq!(body2.velocity, body2_from_redis.velocity);
        assert_eq!(body2.body_type, body2_from_redis.body_type);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_04_load_systems() -> anyhow::Result<()> {
        let system = get_test_system_1();
        let uuid: Uuid;

        {
            let mut galaxy = Galaxy::new("space_build_tests")?;
            galaxy.clear_db()?;
            uuid = galaxy.add_system(system.clone());
            galaxy.save_systems()?;
        }

        let mut galaxy = Galaxy::new("space_build_tests")?;

        galaxy.load_systems()?;

        assert_eq!(1, galaxy.systems.len());

        let loaded_system = galaxy.systems.get(&uuid).unwrap();

        assert_eq!(system.coords, loaded_system.coords);
        assert_eq!(system.offset, loaded_system.offset);
        assert_eq!(system.center_type, loaded_system.center_type);

        let body1 = system.bodies.get(0).unwrap();
        let body2 = system.bodies.get(1).unwrap();

        assert_eq!(2, loaded_system.bodies.len());

        let loaded_body1 = loaded_system.bodies.get(0).unwrap();
        let loaded_body2 = loaded_system.bodies.get(1).unwrap();

        assert_eq!(body1.coords, loaded_body1.coords);
        assert_eq!(body1.velocity, loaded_body1.velocity);
        assert_eq!(body1.body_type, loaded_body1.body_type);

        assert_eq!(body2.coords, loaded_body2.coords);
        assert_eq!(body2.velocity, loaded_body2.velocity);
        assert_eq!(body2.body_type, loaded_body2.body_type);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_05_add_player() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests")?;
        galaxy.clear_db()?;
        let uuid = galaxy.add_player(get_test_player_1());
        assert!(galaxy.players.contains_key(&uuid));
        let player_ref = galaxy.players.get(&uuid).unwrap();

        assert_eq!(Vector3::new(0., 1., 1.), player_ref.coords);
        assert_eq!(
            Uuid::from_str("f599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
            player_ref.current_system_uuid
        );
        assert_eq!(
            Uuid::from_str("f599a2ae-58a8-449f-8007-80de1ea791e9").unwrap(),
            player_ref.own_system_uuid
        );
        assert_eq!("test_nick1", player_ref.nickname);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_06_save_players() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests")?;
        galaxy.clear_db()?;

        let player1 = get_test_player_1();
        let player2 = get_test_player_2();

        let uuid1 = galaxy.add_player(player1);
        let uuid2 = galaxy.add_player(player2);

        galaxy.save_players()?;

        let client = redis::Client::open("redis://127.0.0.1/")?;
        let mut conn = client.get_connection()?;

        let player1_from_redis_json: String =
            conn.get(format!("space_build_tests:player:{uuid1}"))?;

        let player2_from_redis_json: String =
            conn.get(format!("space_build_tests:player:{uuid2}"))?;

        let player1_from_redis = serde_json::from_str::<Player>(&player1_from_redis_json)?;
        let player2_from_redis = serde_json::from_str::<Player>(&player2_from_redis_json)?;

        let player1_ref = galaxy.players.get(&uuid1).unwrap();
        let player2_ref = galaxy.players.get(&uuid2).unwrap();

        assert_eq!(player1_ref.coords, player1_from_redis.coords);
        assert_eq!(player2_ref.coords, player2_from_redis.coords);

        assert_eq!(
            player1_ref.current_system_uuid,
            player1_from_redis.current_system_uuid
        );
        assert_eq!(
            player2_ref.current_system_uuid,
            player2_from_redis.current_system_uuid
        );

        assert_eq!(
            player1_ref.own_system_uuid,
            player1_from_redis.own_system_uuid
        );
        assert_eq!(
            player2_ref.own_system_uuid,
            player2_from_redis.own_system_uuid
        );

        assert_eq!(player1_ref.nickname, player1_from_redis.nickname);
        assert_eq!(player2_ref.nickname, player2_from_redis.nickname);
        Ok(())
    }

    #[test]
    #[serial]
    fn test_07_load_player_by_nickname() -> anyhow::Result<()> {
        let uuid1: Uuid;
        let uuid2: Uuid;
        let player1 = get_test_player_1();
        let player2 = get_test_player_2();

        {
            let mut galaxy = Galaxy::new("space_build_tests")?;
            galaxy.clear_db()?;

            uuid1 = galaxy.add_player(player1.clone());
            uuid2 = galaxy.add_player(player2.clone());

            galaxy.save_players()?;
        }

        let mut galaxy = Galaxy::new("space_build_tests")?;

        assert_eq!(0, galaxy.players.len());
        galaxy.load_player_by_nickname("test_nick1".to_string())?;
        assert_eq!(1, galaxy.players.len());
        galaxy.load_player_by_nickname("test_nick2".to_string())?;
        assert_eq!(2, galaxy.players.len());

        assert!(galaxy.players.contains_key(&uuid1));
        assert!(galaxy.players.contains_key(&uuid2));

        let player1_ref = galaxy.players.get(&uuid1).unwrap();
        let player2_ref = galaxy.players.get(&uuid2).unwrap();

        assert_eq!(player1_ref.coords, player1.coords);
        assert_eq!(player2_ref.coords, player2.coords);

        assert_eq!(player1_ref.current_system_uuid, player1.current_system_uuid);
        assert_eq!(player2_ref.current_system_uuid, player2.current_system_uuid);

        assert_eq!(player1_ref.own_system_uuid, player1.own_system_uuid);
        assert_eq!(player2_ref.own_system_uuid, player2.own_system_uuid);

        assert_eq!(player1_ref.nickname, player1.nickname);
        assert_eq!(player2_ref.nickname, player2.nickname);

        Ok(())
    }
}

#[cfg(test)]
mod tests_gameserver {
    use player_client::PlayerClient;
    use serial_test::serial;

    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_08_all() -> anyhow::Result<()> {
        let (tx, mut game_server) = GameServer::new(Galaxy::new("space_build_tests")?);
        let game_thread = tokio::spawn(async move { game_server.run().await });

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let mut player = PlayerClient::connect("ws://127.0.0.1:2567").await?;
        player.login("test".to_string()).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        tx.send(()).await?;

        game_thread.await??;

        Ok(())
    }
}
