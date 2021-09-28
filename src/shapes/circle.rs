use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Transform2D};

/**
    # Circle

    A Circle is defined as all points with a certain length(radius) from the center point.
*/
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
        let rot = Mat2::from_angle(transform.rotation());

        Aabb {
            extents : Vec2::splat(self.radius),
            position : transform.translation() + rot * self.offset,
        }
    }

    pub fn ray(&self, trans : &Transform2D, ray_origin : Vec2, ray_cast : Vec2) -> Option<f32> {
        let n = ray_cast.normalize();
        let p = n.perp();

        let center = trans.translation() + Mat2::from_angle(trans.rotation()) * self.offset;

        let center_n = n.dot(center);
        let center_p = p.dot(center);
        
        let ray_n = n.dot(ray_origin);
        let ray_p = p.dot(ray_origin);

        if (ray_p - center_p).abs() < self.radius {
            let dis = (self.radius.powi(2) - (ray_p - center_p).powi(2)).sqrt();
            // Why?
            //  We are checking for the edge with the min value(along the n axis) usually,
            //  if it is negative we need to check for the edge with the max value, thus this weird if
            let dis = if center_n - dis < ray_n { center_n + dis } else { center_n - dis } - ray_n;

            if n.dot(ray_cast) > dis && dis > 0.0 {
                Some(dis / n.dot(ray_cast))
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
