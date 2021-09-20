use bevy::prelude::*;
use crate::{
    physics_components::{
        Vel,
        CollisionLayer,
    },
    prelude::CollisionShape,
};

/// Kinematic bodys are practically everything with the `Vel` struct
#[derive(Bundle, Default)]
pub struct KinematicBundle {
    pub vel : Vel,
    pub shape : CollisionShape,
    pub collision_layer : CollisionLayer,
}
