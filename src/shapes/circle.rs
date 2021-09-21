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

    pub fn ray(&self, t : &Transform2D, ro : Vec2, rc : Vec2) -> Option<f32> {
        let n = rc.normalize();
        let p = n.perp();

        let c = t.translation() + self.offset;

        let cn = n.dot(c);
        let cp = p.dot(c);
        
        let rp = p.dot(ro);
        let rn = n.dot(ro);

        if (rp - cp).abs() < self.radius {
            let d = (self.radius.powi(2) - (rp - cp).powi(2)).sqrt();
            // Why?
            //  We are checking for the edge with the min value(along the n axis) usually,
            //  if it is negative we need to check for the edge with the max value, thus this weird if
            let d = if cn - d < rn { cn + d } else { cn - d };

            if n.dot(rc) + rn > d && d > rn {
                Some(1.0 + d / n.dot(rc)) // we want a value between [0.0 - 1.0], and we got a full blown value here
            }
            else {
                None // Ray isnt long enough or the circle is behind the ray 
            }
        }
        else {
            None // No collision can happen because the ray is too far away on the perp axis
        }
    }
}
impl Default for Circle {
    fn default() -> Self {
        Self::new(1.0)
    }
}
