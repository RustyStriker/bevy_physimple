use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Segment;
use super::{Aabb, Shape, Transform2D};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Square {
    /// Offset from the `Transform` transltion component
    pub offset : Vec2,
    /// rotation offset from the `Transform` rotation component
    pub rotation_offset : f32,
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
            rotation_offset : 0.0,
            extents,
        }
    }
    /// Constructs a new square from absolute size(ie. width and height)
    pub fn size(size : Vec2) -> Self {
        Square {
            offset : Vec2::ZERO,
            rotation_offset : 0.0,
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
    /// rotation offset from the `Transform` rotation component
    pub fn with_rotation_offset(
        mut self,
        offset : f32,
    ) -> Self {
        self.rotation_offset = offset;
        self
    }
}
impl Default for Square {
    /// Default square with `extents = Vec2::splat(1.0)`
    fn default() -> Self {
        Self::new(Vec2::splat(1.0))
    }
}
impl Shape for Square {
    fn to_aabb(
        &self,
        transform : Transform2D,
    ) -> Aabb {
        let rot = Mat2::from_angle(transform.rotation);

        // We do the conjugate because if we have extents of (1,1) and we rotate 45deg we get (sqrt_2,0)
        let ex = (rot * (self.extents * transform.scale)).abs(); // we abs in case of a rotation more than 45 degrees
        let ex_con = (rot * (self.extents * Vec2::new(1.0, -1.0) * transform.scale)).abs();
        let extents = ex.max(ex_con);

        Aabb {
            extents,
            position : transform.translation + self.offset,
        }
    }

    fn get_vertex_penetration(
        &self,
        vertex : Vec2,
        transform : Transform2D,
    ) -> (Vec2, bool) {
        let basis = Mat2::from_angle(transform.rotation);
        let basis_inv = basis.inverse();

        let vertex = vertex - (transform.translation + self.offset);
        let vertex = basis_inv * vertex * transform.scale.recip();

        let extents = self.extents;

        // Check that indeed it is between at least 2 parallel edges
        if vertex.x.abs() > extents.x && vertex.y.abs() > extents.y {
            // It is not inside any pair of parallel edges
            // the closest edge will be the one where both signs of the edges will resemle the vertex's signs
            // so it will end up being `extents * vertex.signum()` therefore

            let distance = vertex - (vertex.signum() * extents);

            (distance, false)
        }
        else {
            let to_edge = extents.abs() - vertex.abs();
            let res = if to_edge.x < to_edge.y {
                Vec2::new(to_edge.x * vertex.x.signum(), 0.0)
            }
            else {
                Vec2::new(0.0, to_edge.y * vertex.y.signum())
            };

            let is_pen = vertex.x.abs() < extents.x && vertex.y.abs() < extents.y;

            (basis * res, is_pen)
        }
    }

    fn get_segment_penetration(
        &self,
        segment : super::Segment,
        transform : Transform2D,
        segment_origin : Vec2,
    ) -> f32 {
        let rot = Mat2::from_angle(transform.rotation);
        let extents = rot * (self.extents * transform.scale);
        let extents_con = rot * (self.extents * transform.scale * Vec2::new(1.0, -1.0));

        let center = transform.translation + self.offset;

        let v = [
            center + extents,
            center + extents_con,
            center - extents,
            center - extents_con,
        ];

        let segments = [
            Segment { a: v[0], b: v[1], n: rot * Vec2::new(1.0,0.0) },
            Segment { a: v[1], b: v[2], n: rot * Vec2::new(0.0,-1.0) },
            Segment { a: v[2], b: v[3], n: rot * Vec2::new(-1.0,0.0) },
            Segment { a: v[3], b: v[0], n: rot * Vec2::new(0.0,1.0) },
        ];
        
        let del_origin = segment_origin - transform.translation;
        
        let mut res = f32::INFINITY; // I should probably replace this later on

        for &s in segments.iter() {
            if s.n.dot(del_origin) > 0.0 {
                res = res.min(segment.collide(s).unwrap_or(res));
            }
        }
        res
    }

    fn collide_with_shape(
        &self,
        transform : Transform2D,
        shape : &dyn Shape,
        shape_trans : Transform2D,
    ) -> Option<Vec2> {
        let rot = Mat2::from_angle(transform.rotation);
        let extents = rot * (self.extents * transform.scale);
        let extents_con = rot * (self.extents * transform.scale * Vec2::new(1.0, -1.0));

        let center = transform.translation + self.offset;

        let v = [
            center + extents,
            center + extents_con,
            center - extents,
            center - extents_con,
        ];

        let segments = [
            Segment { a: v[0], b: v[1], n: rot * Vec2::new(1.0,0.0) },
            Segment { a: v[1], b: v[2], n: rot * Vec2::new(0.0,-1.0) },
            Segment { a: v[2], b: v[3], n: rot * Vec2::new(-1.0,0.0) },
            Segment { a: v[3], b: v[0], n: rot * Vec2::new(0.0,1.0) },
        ];
        
        let del_origin = transform.translation - shape_trans.translation;
        
        let mut res = f32::INFINITY; // I should probably replace this later on
        let mut normal = Vec2::ZERO;

        for &s in segments.iter() {
            if s.n.dot(del_origin) < 0.0 {
                let pen = shape.get_segment_penetration(s, shape_trans, transform.translation);
                if pen.abs() < res.abs() {
                    res = pen;
                    normal = s.n;
                }
            }
        }

        if res < 0.0 {
            Some(res * normal)
        }
        else {
            None
        }

    }
}

#[cfg(test)]
mod square_tests {
    use std::f32::consts::PI;
    use super::*;

    #[test]
    fn vertex_rotated_square() {
        let rect = Square::new(Vec2::new(10.0, 5.0));

        let transform = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.5 * PI, // 90 degrees in radians...
            scale : Vec2::splat(1.0),
        };

        let outside = Vec2::new(7.0, 7.0);
        println!("vertex : {:?}", outside);

        let (pen, colliding) = rect.get_vertex_penetration(outside, transform);

        let res = (pen - Vec2::new(-2.0, 0.0)).abs();

        // Use a much higher value of epsilon due to the trigo functions in the rotation calculations having
        //  around 0.0000005 miss
        const EPSILON : f32 = 0.001;
        eprintln!("res {:?}, pen {:?}", res, pen);

        assert!(res.x <= EPSILON && res.y <= EPSILON && !colliding);
    }

    #[test]
    fn vertex_inside_square() {
        let rect = Square::new(Vec2::new(10.0, 5.0));

        let transform = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.5 * PI,
            scale : Vec2::splat(1.0),
        };

        let vertex = Vec2::new(-3.0, 7.0);

        let (pen, coll) = rect.get_vertex_penetration(vertex, transform);

        let res = (pen - Vec2::new(-2.0, 0.0)).abs();

        const EPSILON : f32 = 0.001;
        eprintln!("res {:?} pen {:?}", res, pen);

        assert!(res.x <= EPSILON && res.y <= EPSILON && coll);
    }

    #[test]
    fn segment_collision() {
        let square = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(1.0),
        };

        let trans = Transform2D {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        let seg = Segment {
            a: Vec2::new(0.7,2.0),
            b: Vec2::new(0.7, 0.0),
            n: Vec2::new(-1.0,0.0),
        };

        assert_eq!(square.get_segment_penetration(seg, trans, Vec2::new(1.0,0.0)), -0.3);
    }

    #[test]
    fn square_collision() {
        let square = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(1.0),
        };

        let trans = Transform2D {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        let square2 = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(1.0),
        };

        let trans2 = Transform2D {
            translation: Vec2::new(1.5,0.0),
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        assert_eq!(
            square.collide_with_shape(trans, &square2, trans2),
            Some(Vec2::new(-0.5,0.0))
        );
    }
}
