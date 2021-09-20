use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Transform2D};

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct Capsule {
    /// Offset from the `Transform` translation component
    pub offset : Vec2,

    /// Distance from the center line
    pub radius : f32,

    /// half the length of the center line(so overall height of the capsule will be `2 * (radius + half_height)`)
    pub half_height : f32,
}
impl Capsule {
    pub fn new(height : f32, radius : f32) -> Self {
        Self {
            offset: Vec2::ZERO,
            radius,
            half_height: 0.5 * height,
        }
    }
    /// Offset from the `Transform` translation component
    pub fn with_offset(mut self, offset : Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn aabb(&self, t : &Transform2D) -> Aabb {
        let (a, b) = self.center_line(t);

        let min = a.min(b) - Vec2::splat(self.radius);
        let max = a.max(b) + Vec2::splat(self.radius);

        let extents = (max - min) * 0.5;
        let position = min + extents;

        Aabb { extents, position }
    }

    pub fn ray(&self, t : &Transform2D, ro : Vec2, rc : Vec2) -> Option<f32> {
        // TODO ray for capsules
        None
    }

    pub fn center_line(&self, t : &Transform2D) -> (Vec2, Vec2) {
        let rot = Mat2::from_angle(t.rotation());

        let a = rot * Vec2::new(0.0, self.half_height) + t.translation() + self.offset;
        let b = rot * Vec2::new(0.0, -self.half_height) + t.translation() + self.offset;

        (a, b)
    }

    pub fn sat_normal(&self, t : &Transform2D, vertex : Vec2) -> Vec2 {
        let (a, b) = self.center_line(t);

        let a = a - vertex;
        let b = b - vertex;

        if a.length_squared() < b.length_squared() {
            a.normalize()
        }
        else {
            b.normalize()
        }
    }

    pub fn project(&self, t : &Transform2D, n : Vec2) -> (f32,f32) {
        let (a, b) = self.center_line(t);

        let a = n.dot(a);
        let b = n.dot(b);

        (a.min(b) - self.radius, a.max(b) + self.radius)
    }
}