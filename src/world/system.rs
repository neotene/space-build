use super::{body::Body, temporal::Temporal};
use crate::Result;
use crate::{GalaxyCoordsRepr, GalaxyOffsetRepr};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum CenterType {
    OneStar,
    TwoStars,
    ThreeStars,
    BlackHole,
    NeutronStar,
}

impl Default for CenterType {
    fn default() -> Self {
        CenterType::OneStar
    }
}

impl From<u32> for CenterType {
    fn from(value: u32) -> Self {
        match value {
            0 => CenterType::OneStar,
            1 => CenterType::TwoStars,
            2 => CenterType::ThreeStars,
            3 => CenterType::BlackHole,
            4 => CenterType::NeutronStar,
            _ => panic!("Invalid center type!"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct System {
    pub coords: Vector3<GalaxyCoordsRepr>, // parsec
    pub offset: Vector3<GalaxyOffsetRepr>, // au
    pub center_type: CenterType,
    pub bodies: Vec<Body>,
}

impl System {
    pub fn new(
        coords: Vector3<GalaxyCoordsRepr>,
        offset: Vector3<GalaxyOffsetRepr>,
        center_type: CenterType,
    ) -> Self {
        Self {
            coords,
            offset,
            center_type,
            bodies: Vec::new(),
        }
    }
}

impl Temporal for System {
    fn update(&mut self, delta: f32) -> Result<()> {
        for body in self.bodies.iter_mut() {
            body.update(delta)?;
        }
        Ok(())
    }
}
