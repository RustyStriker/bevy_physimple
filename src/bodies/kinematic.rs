use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    physics_components::{
        velocity::Vel,
        CollisionLayer,
    },
    prelude::{Aabb, BoundingShape, CollisionShape, Obv, Square},
};

/// Kinematic bodys are practically everything with the `Vel` struct
#[derive(Bundle)]
pub struct KinematicBundle {
    pub vel : Vel,
    pub obv : Obv,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}
impl Default for KinematicBundle {
    fn default() -> Self {
        Self {
            vel : Vel::ZERO,
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::splat(1.0))),
            },
            shape : CollisionShape::Square(Square::size(Vec2::splat(1.0))),
            coll_layer : CollisionLayer::new(1, 1),
        }
    }
}
