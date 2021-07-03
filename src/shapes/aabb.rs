use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Aabb {
    pub(crate) extents : Vec2,
    pub(crate) position : Vec2,
}
impl Aabb {
    /// Creates a new AABB from extents(0.5 * absolute size)
    pub fn new(extents : Vec2) -> Aabb {
        Aabb {
            extents : extents.abs(),
            position : Vec2::ZERO,
        }
    }
    /// Creates a new AABB object from absolute size
    pub fn size(size : Vec2) -> Aabb {
        Aabb {
            extents : size.abs() * 0.5,
            position : Vec2::ZERO,
        }
    }
}
impl Default for Aabb {
    fn default() -> Self {
        Self::new(Vec2::ZERO)
    }
}

/// Temp struct for aabb collision event
#[derive(Clone, Debug)]
pub struct AABBCollisionEvent {
    pub entity_a : Entity,
    pub entity_b : Entity,
    /// Penetration of a in b, can get normal out of it
    pub penetration : Vec2,
    /// If the collision happened with a static body
    pub with_static : bool,
}
