use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::CollisionLayer;

#[derive(Bundle)]
pub struct RayCastBundle {
    ray: RayCast,
    collision_layer: CollisionLayer,
}

/// TODO raycast explanation...
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RayCast {
    /// Offset from the Transform object
    pub offset : Vec2,

    /// The position relative to the ray's origin
    pub cast : Vec2,

    /// Whether to try and collide with static objects as well(defaults to true)
    pub collide_with_static : bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) collision : Option<RayCastCollision>,
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct RayCastCollision {
    /// Position in global space
    pub collision_point : Vec2,
    /// Entity it collides with
    pub entity : Entity,
    /// Whether the entity is a statcibody or not
    pub is_static : bool,
}

impl RayCast {
    /// Creates a new raycast object
    ///
    /// offset from the transform of the raycast entity
    ///
    /// cast - the direction(and length) the ray shoots
    pub fn new(cast : Vec2) -> Self {
        RayCast {
            offset : Vec2::ZERO,
            cast : cast,
            collide_with_static : true,
            collision : None,
        }
    }
    pub fn with_offset(
        mut self,
        offset : Vec2,
    ) -> Self {
        self.offset = offset;
        self
    }
    /// Whether to try and collide with static objects as well(defaults to true)
    pub fn with_static(
        mut self,
        collide_with_static : bool,
    ) -> Self {
        self.collide_with_static = collide_with_static;
        self
    }

    pub fn get_collision(&self) -> Option<RayCastCollision> {
        self.collision
    }
}