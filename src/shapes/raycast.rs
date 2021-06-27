use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Raycasts work in
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct RayCast2D {
    /// Offset from the Transform object
    pub offset: Vec2,

    /// The Direction the ray shoots and the length
    pub cast: Vec2,

    /// Mask of the ray, will check only for objects by `self.mask & object.layer > 0`
    pub mask: u8,

    /// Whether to try and collide with static objects as well(defaults to true)
    pub collide_with_static: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) collision: Option<RayCastCollision>,
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct RayCastCollision {
    pub collision_point: Vec2,
    pub entity: Entity,
    pub is_static: bool,
}

impl RayCast2D {
    /// Creates a new raycast object
    ///
    /// offset from the transform of the raycast entity
    ///
    /// cast - the direction(and length) the ray shoots
    pub fn new(cast: Vec2) -> Self {
        RayCast2D {
            offset: Vec2::ZERO,
            cast,
            mask: 1,
            collide_with_static: true,
            collision: None,
        }
    }
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
    /// Mask of the ray, will check only for objects by `self.mask & object.layer > 0`
    pub fn with_mask(mut self, mask: u8) -> Self {
        self.mask = mask;
        self
    }
    /// Whether to try and collide with static objects as well(defaults to true)
    pub fn with_static(mut self, collide_with_static: bool) -> Self {
        self.collide_with_static = collide_with_static;
        self
    }

    pub fn get_collision(&self) -> Option<RayCastCollision> {
        self.collision
    }
}
