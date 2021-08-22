//! All the different components which describe a physical body

pub mod velocity;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct CollisionLayer {
    pub mask : u8,
    pub layer : u8,
}
impl CollisionLayer {
    pub fn new(
        mask : u8,
        layer : u8,
    ) -> Self {
        Self { mask, layer }
    }
    pub fn overlap(
        &self,
        other : &CollisionLayer,
    ) -> bool {
        (self.mask & other.layer) | (self.layer & other.mask) != 0
    }
}
