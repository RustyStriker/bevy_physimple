//! All the different components which describe a physical body

mod velocity;
mod transform2d;
pub use transform2d::Transform2D;
pub use velocity::Vel;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct CollisionLayer {
    pub mask : u8,
    pub layer : u8,
}

impl Default for CollisionLayer {
    fn default() -> Self {
        Self { mask: 1, layer: 1 }
    }
}
impl CollisionLayer {
    pub fn new(
        mask : u8,
        layer : u8,
    ) -> Self {
        Self { mask, layer }
    }
    /// Checks if 2 `CollisionLayer`s should collide with each other
    pub fn overlap(
        &self,
        other : &CollisionLayer,
    ) -> bool {
        (self.mask & other.layer) | (self.layer & other.mask) != 0
    }
}

