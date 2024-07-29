use super::temporal::Temporal;
use crate::Result;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BodyType {
    Planet,
    Asteroid,
    Station,
}

impl From<u32> for BodyType {
    fn from(value: u32) -> Self {
        match value {
            0 => BodyType::Planet,
            1 => BodyType::Asteroid,
            2 => BodyType::Station,
            _ => panic!("Invalid body type!"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Body {
    pub body_type: BodyType,
    pub coords: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl Body {
    pub fn new(body_type: BodyType, coords: Vector3<f32>) -> Body {
        Self {
            body_type,
            coords,
            velocity: Vector3::default(),
        }
    }
}

impl Temporal for Body {
    fn update(&mut self, delta: f32) -> Result<()> {
        self.coords += self.velocity * delta;
        Ok(())
    }
}
