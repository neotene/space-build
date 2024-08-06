use crate::Result;

pub trait Temporal {
    fn update(&mut self, delta: f32) -> Result<()>;
}
