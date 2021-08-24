use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Axis aligned bounding box
#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Aabb {
    pub extents : Vec2,
    pub position : Vec2,
}
impl Aabb {
    /// Creates a new AABB from extents(0.5 * absolute size)
    pub fn new(extents : Vec2, position : Vec2) -> Aabb {
        Aabb {
            extents : extents.abs(),
            position
        }
    }
    /// Creates a new AABB object from absolute size
    pub fn size(size : Vec2, position : Vec2) -> Aabb {
        Aabb {
            extents : size.abs() * 0.5,
            position,
        }
    }
}
