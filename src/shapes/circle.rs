use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Shape, Transform2D};

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
}
impl Default for Circle {
    fn default() -> Self {
        Self::new(1.0)
    }
}
impl Shape for Circle {
    fn to_aabb(
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

    fn collide_vertex(
        &self,
        vertex : Vec2,
        transform : &Transform2D,
    ) -> (Vec2, bool) {
        let vertex = vertex - (transform.translation() + self.offset);

        // Shrink down the vertex based on scale
        let vertex = vertex * transform.scale().recip();

        let distance = vertex.length();

        let normal = vertex / distance; // Basically normalizing the vector

        (normal * (self.radius - distance), distance < self.radius) // Return the penetration value
    }

    fn collide(
        &self,
        transform : &Transform2D,
        shape : &dyn Shape,
        shape_trans : &Transform2D,
    ) -> Option<Vec2> {
        let center = transform.translation() + self.offset;

        let (dis, is_pen) = shape.collide_vertex(center, &shape_trans);

        if is_pen {
            let normal = dis.normalize();
            let pen = dis + normal * self.radius;

            Some(pen)
        }
        else {
            let dis_len = dis.length();

            if dis_len < f32::EPSILON {
                return Some(center - shape_trans.translation());
            }

            // calculate the distance to the shape
            let pen = (self.radius - dis_len) * dis / dis_len;

            if dis_len < self.radius {
                Some(-pen)
            }
            else {
                None
            }
        }
    }

    fn collide_segment(
        &self,
        segment : super::Segment,
        transform : &Transform2D,
    ) -> f32 {
        let (n, p) = segment.collide_point(transform.translation() + self.offset);

        // check we are actually close enough to the circle
        if (n.powi(2) + p.powi(2)) < self.radius.powi(2) {
            let b = (self.radius.powi(2) - p.powi(2)).sqrt(); // this is pretty much c^2 = a^2 + b^2 -> b^2 = c^2 - a^2
            (n.abs() - b).copysign(n) // b is the "distance" between the point `center + p * normal.perp()` to the "bottom" of the circle
        }
        else {
            f32::INFINITY
        }
    }

    fn collide_ray(&self, transform : &Transform2D, ray : (Vec2, f32), ray_origin : Vec2) -> Option<f32> {
        todo!();
    }
}

#[cfg(test)]
mod circle_tests {
    use crate::prelude::Segment;

    use super::*;

    // Use a much higher value of epsilon due to the trigo functions in the rotation calculations having
    //  around 0.0000005 miss
    const EPSILON : f32 = 0.001;
    #[test]
    fn vetrex() {
        let c = Circle {
            offset : Vec2::ZERO,
            radius : 10.0,
        };

        let tc = Transform2D::new(
            Vec2::ZERO,
            0.0,
            Vec2::splat(1.0),
        );

        let v1 = Vec2::new(10.0, 10.0);

        let c1 = c.collide_vertex(v1, &tc);
        assert!(!c1.1); // Check it is outside
        println!("{:?}", c1.0);
        // the result should be -(10 - 5 * sqrt(2))
        assert!((c1.0 - Vec2::splat(-10.0 + 5.0 * 2.0_f32.sqrt())).length() < EPSILON);

        let v2 = Vec2::new(0.0, -5.0);
        let c2 = c.collide_vertex(v2, &tc);

        assert!(c2.1); // make sure its inside

        assert!((c2.0 - Vec2::new(0.0, -5.0)).length() < EPSILON);
    }

    #[test]
    fn segment() {
        let c = Circle {
            offset : Vec2::ZERO,
            radius : 10.0,
        };

        let tc = Transform2D::new(
            Vec2::ZERO,
            0.0,
            Vec2::splat(1.0),
        );

        let s1 = Segment {
            a : Vec2::new(5.0, 5.0),
            b : Vec2::new(10.0, 5.0),
            n : Vec2::new(0.0, -1.0),
        };

        let c1 = c.collide_segment(s1, &tc);
        println!("{}", c1);

        assert!((c1 - (5.0 * 3.0_f32.sqrt() - 5.0)).abs() < EPSILON);
    }
}

// input: 2 arrays sorted(rising)
// output:
