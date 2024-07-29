mod error;
mod game;
mod world;

pub use error::Error;
pub use game::ClientMessage;
pub use game::Login;
pub use game::SpaceBuildGame;
pub use world::galaxy::Galaxy;
pub type Result<T> = std::result::Result<T, Error>;
pub type GalaxyCoordsRepr = i16; // parsec
pub type GalaxyOffsetRepr = i32; // au
pub type SystemCoordsRepr = f32; // meter

#[cfg(test)]
mod tests_galaxy {

    use nalgebra::Vector3;
    use redis::Commands;
    use world::{
        body::{Body, BodyType},
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

    #[test]
    fn add_system() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests".to_string())?;
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
    fn save_systems() -> anyhow::Result<()> {
        let mut galaxy = Galaxy::new("space_build_tests".to_string())?;
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
}
