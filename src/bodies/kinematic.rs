use bevy::prelude::*;
use crate::{
    physics_components::{
        Vel,
        CollisionLayer,
    },
    prelude::{CollisionShape, Square},
};

/// Kinematic bodys are practically everything with the `Vel` struct
#[derive(Bundle)]
pub struct KinematicBundle {
    pub vel : Vel,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}
impl Default for KinematicBundle {
    fn default() -> Self {
        Self {
            vel : Vel::ZERO,
            shape : CollisionShape::Square(Square::size(Vec2::splat(1.0))),
            coll_layer : CollisionLayer::new(1, 1),
        }
    }
}
