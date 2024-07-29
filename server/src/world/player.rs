use crate::SystemCoordsRepr;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub coords: Vector3<SystemCoordsRepr>,
    pub nickname: String,
    pub own_system_uuid: Uuid,
    pub current_system_uuid: Uuid,
}

impl Player {
    pub fn new(coords: Vector3<SystemCoordsRepr>, nickname: String, system_uuid: Uuid) -> Self {
        Self {
            coords,
            nickname,
            own_system_uuid: system_uuid,
            current_system_uuid: system_uuid,
        }
    }
}
