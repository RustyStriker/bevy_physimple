use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    physics_components::{
        physical_properties::{FrictionMult, Mass},
        velocity::{TerVel, Vel},
        CollisionLayer,
    },
    prelude::{Aabb, BoundingShape, CollisionShape, Obv, Square},
};

#[derive(Bundle)]
pub struct KinematicBundle {
    pub kin : KinematicBody2D,
    pub vel : Vel,
    pub mass : Mass,
    pub obv : Obv,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
    pub terminal_vel : TerVel,
    pub friction : FrictionMult,
}
impl Default for KinematicBundle {
    fn default() -> Self {
        Self {
            kin : KinematicBody2D::default(),
            vel : Vel::ZERO,
            mass : Mass::new(1.0),
            obv : Obv {
                offset : Vec2::ZERO,
                shape : BoundingShape::Aabb(Aabb::size(Vec2::splat(1.0))),
            },
            shape : CollisionShape::Square(Square::size(Vec2::splat(1.0))),
            coll_layer : CollisionLayer::new(1, 1),
            terminal_vel : TerVel::default(),
            friction : FrictionMult(1.0),
        }
    }
}

/// KinematicBody for 2D physics, for moving objects
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct KinematicBody2D {
    /// Some(normal) if on_floor
    pub(crate) on_floor : Option<Vec2>,
    /// Some(normal) if on_wall
    pub(crate) on_wall : Option<Vec2>,
    /// Some(normal) if on_ceil
    pub(crate) on_ceil : Option<Vec2>,
}

impl KinematicBody2D {
    /// Returns a new 'default' KinematicBody2D
    pub fn new() -> Self {
        KinematicBody2D {
            on_floor : None,
            on_wall : None,
            on_ceil : None,
        }
    }
    /// Get Floor normal if body is on floor
    pub fn on_floor(&self) -> Option<Vec2> {
        self.on_floor
    }
    /// Get wall normal if body is touching a wall
    pub fn on_wall(&self) -> Option<Vec2> {
        self.on_wall
    }
    /// Get ceilling normal if body is touching a ceiling
    pub fn on_ceil(&self) -> Option<Vec2> {
        self.on_ceil
    }
}

impl Default for KinematicBody2D {
    fn default() -> Self {
        Self::new()
    }
}
