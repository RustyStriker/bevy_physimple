use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Transform2D;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Square {
    /// Offset from the `Transform` transltion component
    pub offset : Vec2,
    /// Square's extents
    ///
    /// `extents = Vec2::new(half width, half height)`
    pub extents : Vec2,
}
impl Square {
    /// Constructs a new square
    pub fn new(extents : Vec2) -> Self {
        Square {
            offset : Vec2::ZERO,
            extents,
        }
    }
    /// Constructs a new square from absolute size(ie. width and height)
    pub fn size(size : Vec2) -> Self {
        Square {
            offset : Vec2::ZERO,
            extents : size * 0.5,
        }
    }
    /// Offset from the `Transform` transltion component
    pub fn with_offset(
        mut self,
        offset : Vec2,
    ) -> Self {
        self.offset = offset;
        self
    }
}
impl Default for Square {
    /// Default square with `extents = Vec2::splat(1.0)`
    fn default() -> Self {
        Self::new(Vec2::splat(1.0))
    }
}

impl super::SAT for Square {
    fn get_normals(&self, trans : &Transform2D) -> Vec<Vec2> {
        let rot = Mat2::from_angle(trans.rotation());

        Vec::from([rot * Vec2::Y, rot * Vec2::X])
    }

    fn project(&self, trans : &Transform2D, normal : Vec2) -> (f32,f32) {
        let rot = Mat2::from_angle(trans.rotation());
        let offset = rot * self.offset;

        let verts = [
            Vec2::new(1.0,1.0),
            Vec2::new(1.0,-1.0),
            Vec2::new(-1.0,1.0),
            Vec2::new(-1.0,-1.0),
        ];

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for v in verts {
            let v = rot * (v * self.extents) + trans.translation() + offset;
            let proj = v.dot(normal);

            min = min.min(proj);
            max = max.max(proj);
        }

        (min, max)
    }

    fn get_closest_vertex(&self, trans : &Transform2D, vertex : Vec2) -> Vec2 {
        let rot = Mat2::from_angle(trans.rotation());
        let offset = rot * self.offset;
    
        let verts = [
            Vec2::new(1.0,1.0),
            Vec2::new(1.0,-1.0),
            Vec2::new(-1.0,1.0),
            Vec2::new(-1.0,-1.0),
        ];

        let mut min_l = f32::INFINITY;
        let mut closest = Vec2::ZERO;

        for v in verts {
            let v = rot * (v * self.extents) + trans.translation() + offset;
        
            let l = (v - vertex).length_squared();
            if l < min_l {
                min_l = l;
                closest = v;
            }
        }

        closest
    }

    fn ray(&self, trans : &Transform2D, ro : Vec2, rc :  Vec2) -> Option<f32> {
        let rot = Mat2::from_angle(-trans.rotation());

        // IDEA: rotate the ray (the opposite direction) and then you can do simple ray vs aabb collision
        let t = rot * (trans.translation()) + self.offset; // offset should not be rotated here

        let ro = rot * ro;
        let rc = rot * rc;

        let smin = t - self.extents;
        let smax = t + self.extents;

        // if one of the cast components is 0.0, make sure we are in the bounds of that axle
        // Why?
        //      We do this explicit check because the raycast formula i used doesnt handle cases where one of the components is 0
        //       as it would lead to division by 0(thus errors) and the `else NAN` part will make it completly ignore the collision
        //       on that axle
        if rc.x.abs() < f32::EPSILON && !(smin.x <= ro.x && smax.x >= ro.x) {
            return None; // if it doesnt collide on the X axle terminate it early
        }
        if rc.y.abs() < f32::EPSILON && !(smin.y <= ro.y && smax.y >= ro.y) {
            return None; // if it doesnt collide on the X axle terminate it early
        }

        // The if else's are to make sure we dont divide by 0.0, because if the ray is parallel to one of the axis
        // it will never collide(thus division by 0.0)
        let xmin = if rc.x != 0.0 { (smin.x - ro.x) / rc.x } else { f32::NAN };
        let xmax = if rc.x != 0.0 { (smax.x - ro.x) / rc.x } else { f32::NAN };
        let ymin = if rc.y != 0.0 { (smin.y - ro.y) / rc.y } else { f32::NAN };
        let ymax = if rc.y != 0.0 { (smax.y - ro.y) / rc.y } else { f32::NAN };
        
        let min = (xmin.min(xmax)).max(ymin.min(ymax));
        let max = (xmin.max(xmax)).min(ymin.max(ymax));

        if max < 0.0 || min > max || min > 1.0 {
            // either the shape is entirely behind us
            //      or we are not colliding at all
            //      or the shape is too far away
            None
        }
        else if min < 0.0 {
            // we are inside the shape
            Some(max)
        }
        else {
            // normal collision(gosh that was hard)
            Some(min)
        }
    }
}

#[cfg(test)]
mod square_tests {
    use crate::prelude::SAT;

    use super::*;
    const EPSILON : f32 = 0.0001;

    #[test]
    fn square_ray() {
        let s = Square {
            offset: Vec2::ZERO,
            extents: Vec2::splat(10.0),
        };

        let ts = Transform2D::new(
            Vec2::ZERO,
            0.0,
            Vec2::splat(1.0),
        );

        // TEST 1 - simple collision
        let r1 = Vec2::new(10.0,0.0);
        let t1 = Vec2::new(-16.0,-5.0);

        let c1 = s.ray(&ts, t1, r1);
        assert!(c1.is_some());
        // should be Vec2(6.0,0.0) so 0.6
        println!("{:?}", c1);
        assert!((c1.unwrap() - 0.6).abs() < EPSILON);

        // TEST 2 - no collision
        let r2 = Vec2::new(1.0,1.0);
        let t2 = Vec2::new(100.0,100.0);

        let c2 = s.ray(&ts, t2, r2);

        assert!(c2.is_none());
    }
}
