use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Transform2D};

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct Circle {
    /// Offset from the `Transform` translation component
    pub offset : Vec2,

    /// Circle's radius
    pub radius : f32,
}
impl Circle {
    pub fn new(radius : f32) -> Self {
        Circle {
            offset : Vec2::ZERO,
            radius,
        }
    }
    /// Offset from the `Transform` translation component
    pub fn with_offset(
        mut self,
        offset : Vec2,
    ) -> Self {
        self.offset = offset;
        self
    }

    pub fn aabb(
        &self,
        transform : &Transform2D,
    ) -> Aabb {
        // rotate the scale
        let basis = Mat2::from_angle(transform.rotation());
        let scale = basis * transform.scale();

        Aabb {
            extents : scale * self.radius,
            position : transform.translation() + self.offset,
        }
    }

    pub fn ray(&self, _t : &Transform2D, _ro : Vec2, _rc : Vec2) -> Option<f32> {
        // TODO ray for circles
        None
    }
}
impl Default for Circle {
    fn default() -> Self {
        Self::new(1.0)
    }
}
