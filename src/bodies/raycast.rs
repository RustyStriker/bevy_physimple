use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::CollisionLayer;

#[derive(Bundle, Default)]
pub struct RayCastBundle {
    pub ray: RayCast,
    pub collision_layer: CollisionLayer,
}

/**
    # RayCast

    A raycast represents a line in space, from its origin(the `GlobalTransform`, with the raycast offset) to `cast`(relative to the origin).

    It is used to check which object is the closest along the given ray.

    ### Note - Multiple checks per frame

    If you need to check for multiple rays per frame, you might find it useful to clone and modify the `ray_phase` system under `normal_coll.rs`
    as currently there is no built in support for it.
*/
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RayCast {
    /// Offset from the Transform object
    pub offset: Vec2,

    /// The position relative to the ray's origin
    pub cast: Vec2,

    /// Whether to try and collide with static objects as well(defaults to true)
    pub collide_with_static: bool,

    #[serde(skip_serializing, skip_deserializing)]
    pub collision: Option<RayCastCollision>,
}
impl Default for RayCast {
    fn default() -> Self {
        Self::new(Vec2::new(0.0,-100.0))
    }
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct RayCastCollision {
    /// The position in global space of the collision
    pub collision_point: Vec2,
    /// The entity which the ray collides with
    pub entity: Entity,
    /// Whether the entity is a statcibody or not - will always be `false` if `Ray.collides_with_static` is false
    pub is_static: bool,
}

impl RayCast {
    /// Creates a new raycast object
    ///
    /// offset from the transform of the raycast entity
    ///
    /// cast - the direction(and length) the ray shoots
    pub fn new(cast: Vec2) -> Self {
        RayCast {
            offset: Vec2::ZERO,
            cast,
            collide_with_static: true,
            collision: None,
        }
    }
    /// Offsets the raycast by `offset` relative to the `Transform` component on the entity
    pub fn with_offset(
        mut self,
        offset: Vec2,
    ) -> Self {
        self.offset = offset;
        self
    }
    /// Whether to try and collide with static objects as well(defaults to true)
    pub fn with_static(
        mut self,
        collide_with_static: bool,
    ) -> Self {
        self.collide_with_static = collide_with_static;
        self
    }

    pub fn get_collision(&self) -> Option<RayCastCollision> {
        self.collision
    }
}
