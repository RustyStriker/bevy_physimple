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
        transform : Transform2D,
    ) -> Aabb {
        // rotate the scale
        let basis = Mat2::from_angle(transform.rotation);
        let scale = basis * transform.scale;

        Aabb {
            extents : scale * self.radius,
            position : transform.translation + self.offset,
        }
    }

    fn get_vertex_penetration(
        &self,
        vertex : Vec2,
        transform : Transform2D,
    ) -> (Vec2, bool) {
        let vertex = vertex - (transform.translation + self.offset);

        // Shrink down the vertex based on scale
        let vertex = vertex * transform.scale.recip();

        let distance = vertex.length();

        let normal = vertex / distance; // Basically normalizing the vector

        (normal * (self.radius - distance), distance < self.radius) // Return the penetration value
    }

    fn collide_with_shape(
        &self,
        transform : Transform2D,
        shape : &dyn Shape,
        shape_trans : Transform2D,
    ) -> Option<Vec2> {
        let center = transform.translation + self.offset;

        let (dis, is_pen) = shape.get_vertex_penetration(center, shape_trans);

        if is_pen {
            let normal = dis.normalize();
            let pen = dis + normal * self.radius;

            Some(pen)
        }
        else {
            let dis_len = dis.length();

            if dis_len < f32::EPSILON {
                return Some(center - shape_trans.translation);
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

    fn get_segment_penetration(
        &self,
        segment : super::Segment,
        transform : Transform2D,
        _ : Vec2,
    ) -> f32 {
        let (n, p) = segment.collide_point(transform.translation + self.offset);

        // check we are actually close enough to the circle
        if (n.powi(2) + p.powi(2)) < self.radius.powi(2) {
            n - self.radius
        }
        else {
            f32::INFINITY
        }
    }
}

#[cfg(test)]
mod circle_tests {
    use super::*;

    #[test]
    fn vertex_testing() {
        let circle = Circle::new(5.0).with_offset(Vec2::new(5.0, 0.0));

        let transform = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.0,
            scale : Vec2::splat(1.0),
        };

        let vertex_a = Vec2::new(0.0, 5.0); // Shouldnt be inside

        let (_, a_coll) = circle.get_vertex_penetration(vertex_a, transform);
        // vertex shouldnt be inside the thing
        assert!(!a_coll);

        let vertex_b = Vec2::new(2.0, 0.0); // should be inside

        let (b_pen, b_coll) = circle.get_vertex_penetration(vertex_b, transform);

        assert!(b_coll);
        assert_eq!(b_pen, Vec2::new(-2.0, 0.0));
    }
}
