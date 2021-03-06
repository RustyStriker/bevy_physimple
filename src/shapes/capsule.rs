use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Transform2D};

/**
    # Capsule

    A Capsule can be defined as all points with a given length(radius) from a certain line, 
    capsule here is defined using the length of the middle line(height) and a radius.
*/
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct Capsule {
    /// Offset from the `Transform` translation component
    pub offset: Vec2,

    /// Distance from the center line
    pub radius: f32,

    /// half the length of the center line(so overall height of the capsule will be `2 * (radius + half_height)`)
    pub half_height: f32,
}
impl Capsule {
    pub fn new(height: f32, radius: f32) -> Self {
        Self {
            offset: Vec2::ZERO,
            radius,
            half_height: 0.5 * height,
        }
    }
    /// Offset from the `Transform` translation component
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn aabb(&self, t: &Transform2D) -> Aabb {
        let (a, b) = self.center_line(t);

        let min = a.min(b) - Vec2::splat(self.radius);
        let max = a.max(b) + Vec2::splat(self.radius);

        let extents = (max - min) * 0.5;
        let position = min + extents;

        Aabb { extents, position }
    }

    pub fn ray(&self, trans: &Transform2D, ray_origin: Vec2, ray_cast: Vec2) -> Option<f32> {
        let (a,b) = self.center_line(trans);
        // Make sure the ray is indeed in the correct height
        let n = ray_cast.normalize();
        let p = n.perp();
        
        let ap = p.dot(a);
        let bp = p.dot(b);
        let minp = ap.min(bp);
        let maxp = ap.max(bp);

        let rp = p.dot(ray_origin); // p.dot(rc) should be equal p.dot(ro) since we are working on the perp axis to rc
        let rp = if rp < minp { rp - minp } else if rp > maxp { rp - maxp } else { 0.0 };

        let rc_len = n.dot(ray_cast);

        if rp.abs() < f32::EPSILON {
            // practically 0, do ray v line(square-ish)
            let yp = (rp - ap) / (bp - ap); // Should be in [0,1]
            let yn = n.dot(yp * (b - a) + a) - n.dot(ray_origin);
            let dis = if yn - self.radius < 0.0 { yn + self.radius } else { yn - self.radius };

            if dis < rc_len && dis > 0.0 {
                Some(dis / rc_len)
            }
            else {
                None // either we are behind the ray, or too far
            }
        }
        else if rp.abs() < self.radius {
            let center = if rp.is_sign_positive() {
                if ap > bp {
                    n.dot(a)
                }            
                else {
                    n.dot(b)
                }    
             } 
             else if ap < bp {
                n.dot(a)
            }
            else {
                n.dot(b)
             } - n.dot(ray_origin);

            // this is a ray v circle kind of thing, but modified a bit
            // we are indeed in range for the circle
            let dis = (self.radius.powi(2) - rp.powi(2)).sqrt();

            // Why?
            //  We are checking for the edge with the min value(along the n axis) usually,
            //  if it is negative we need to check for the edge with the max value, thus this weird if
            let dis = if center - dis < 0.0 { center + dis } else { center - dis };

            if rc_len > dis && dis > 0.0 {
                Some(dis / rc_len)
            }
            else {
                None
            }
        }
        else {
            // ray is too far up/down to hit the capsule
            None
        }
    }

    pub fn center_line(&self, t: &Transform2D) -> (Vec2, Vec2) {
        let rot = Mat2::from_angle(t.rotation());

        let a = rot * Vec2::new(0.0, self.half_height) + t.translation() + rot * self.offset;
        let b = rot * Vec2::new(0.0, -self.half_height) + t.translation() + rot * self.offset;

        (a, b)
    }

    pub fn sat_normal(&self, t: &Transform2D, vertex: Vec2) -> Vec2 {
        let (a, b) = self.center_line(t);
        let n = a - b;

        let an = n.dot(a);
        let bn = n.dot(b);
        let vn = n.dot(vertex);

        if vn > an.min(bn) && vn < an.max(bn) {
            Mat2::from_angle(t.rotation()) * Vec2::X
        }
        else {
            let a = a - vertex;
            let b = b - vertex;

            if a.length_squared() < b.length_squared() {
                a.normalize()
            }
            else {
                b.normalize()
            }
        }
    }

    pub fn project(&self, t: &Transform2D, n: Vec2) -> (f32,f32) {
        let (a, b) = self.center_line(t);

        let a = n.dot(a);
        let b = n.dot(b);

        (a.min(b) - self.radius, a.max(b) + self.radius)
    }
}