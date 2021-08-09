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
        }
    }

    fn collide_vertex(
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

            let distance = (vertex.signum() * extents) - vertex;

            (basis * distance, false)
        }
        else {
            let to_edge = extents.abs() - vertex.abs();
            let res = if to_edge.x > 0.0 && to_edge.y > 0.0 {
                // if both of them are positive, we have penetration and we want the "fastest" way out
                if to_edge.x < to_edge.y {
                    Vec2::new(to_edge.x * vertex.x.signum(), 0.0)
                }
                else {
                    Vec2::new(0.0, to_edge.y * vertex.y.signum())
                }
            }
            else {
                // if 1 of them is negative then there is no penetration and we want the negative value
                if to_edge.x < 0.0 {
                    Vec2::new(to_edge.x * vertex.x.signum(),0.0)
                }
                else {
                    Vec2::new(0.0, to_edge.y * vertex.y.signum())
                }
            };

            let is_pen = vertex.x.abs() < extents.x && vertex.y.abs() < extents.y;

            (basis * res, is_pen)
        }
    }

    fn collide_segment(
        &self,
        segment : super::Segment,
        transform : Transform2D,
        _ : Vec2, // TODO remove when fully realising its useless
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
            Segment {
                a : v[0],
                b : v[1],
                n : rot * Vec2::new(1.0, 0.0),
            },
            Segment {
                a : v[1],
                b : v[2],
                n : rot * Vec2::new(0.0, -1.0),
            },
            Segment {
                a : v[2],
                b : v[3],
                n : rot * Vec2::new(-1.0, 0.0),
            },
            Segment {
                a : v[3],
                b : v[0],
                n : rot * Vec2::new(0.0, 1.0),
            },
        ];

        let mut res = f32::INFINITY; // I should probably replace this later on

        for &s in segments.iter() {
            if s.n.dot(segment.n) < 0.0 {
                let seg = segment.collide(s);
                if let Some(s) = seg {
                    if s.abs() < res.abs() {
                        res = s;
                    }
                }
            }
        }
        res
    }

    fn collide(
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
            Segment {
                a : v[0],
                b : v[1],
                n : rot * Vec2::new(1.0, 0.0),
            },
            Segment {
                a : v[1],
                b : v[2],
                n : rot * Vec2::new(0.0, -1.0),
            },
            Segment {
                a : v[2],
                b : v[3],
                n : rot * Vec2::new(-1.0, 0.0),
            },
            Segment {
                a : v[3],
                b : v[0],
                n : rot * Vec2::new(0.0, 1.0),
            },
        ];

        let (dis, _) = shape.collide_vertex(center, shape_trans);

        let mut res = f32::INFINITY; // I should probably replace this later on
        let mut normal = Vec2::ZERO;

        for &s in segments.iter() {
            if s.n.dot(dis) >= 0.0 {
                let pen = shape.collide_segment(s, shape_trans, transform.translation);
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
    use super::*;
    use std::f32::consts::PI;
    // Use a much higher value of epsilon due to the trigo functions in the rotation calculations having
    //  around 0.0000005 miss
    const EPSILON : f32 = 0.001;

    #[test]
    fn vertex_no_rot() {
        let rect = Square::new(Vec2::new(10.0, 5.0));

        let transform = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.0,
            scale : Vec2::splat(1.0),
        };

        let inside = Vec2::new(7.0,1.0);

        let (inside_res, inside_pen) = rect.collide_vertex(inside, transform);
        assert!(inside_pen && (inside_res - Vec2::new(3.0,0.0)).length() < EPSILON);
        
        let outside = Vec2::new(-3.0,7.0);

        let (out_res, out_pen) = rect.collide_vertex(outside, transform);
        assert!(!out_pen && (out_res - Vec2::new(0.0,-2.0)).length() < EPSILON);

    }

    #[test]
    fn vertex_rot() {
        let rect = Square::new(Vec2::new(10.0, 5.0));

        let transform = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.25 * PI,
            scale : Vec2::splat(1.0),
        };

        // Basically the point (7.0,0.0) rotated 0.25 * PI...
        let inside = Mat2::from_angle(0.25 * PI) * Vec2::new(7.0,0.0);

        let (ins_res, ins_pen) = rect.collide_vertex(inside, transform);
        assert!(ins_pen && (ins_res - Mat2::from_angle(0.25 * PI) * Vec2::new(3.0,0.0)).length() < EPSILON);

        let outside = Mat2::from_angle(0.25 * PI) * Vec2::new(-11.0, -6.0);

        let (out_res, out_pen) = rect.collide_vertex(outside, transform);
        println!("r {:?}", out_res);
        assert!(!out_pen && (out_res - Mat2::from_angle(0.25 * PI) * Vec2::new(1.0,1.0)).length() < EPSILON);
    }

    #[test]
    fn segment_collision() {
        let square = Square {
            offset : Vec2::ZERO,
            rotation_offset : 0.0,
            extents : Vec2::splat(1.0),
        };

        let trans = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.0,
            scale : Vec2::splat(1.0),
        };

        let seg = Segment {
            a : Vec2::new(0.7, 2.0),
            b : Vec2::new(0.7, 0.0),
            n : Vec2::new(-1.0, 0.0),
        };

        assert_eq!(
            square.collide_segment(seg, trans, Vec2::new(1.0, 0.0)),
            -0.3
        );
    }

    #[test]
    fn collision_no_rotation() {
        let square = Square {
            offset : Vec2::ZERO,
            rotation_offset : 0.0,
            extents : Vec2::splat(1.0),
        };

        let trans = Transform2D {
            translation : Vec2::ZERO,
            rotation : 0.0,
            scale : Vec2::splat(1.0),
        };

        let square2 = Square {
            offset : Vec2::ZERO,
            rotation_offset : 0.0,
            extents : Vec2::splat(1.0),
        };

        let trans2 = Transform2D {
            translation : Vec2::new(1.5, 0.0),
            rotation : 0.0,
            scale : Vec2::splat(1.0),
        };

        assert_eq!(
            square.collide(trans, &square2, trans2),
            Some(Vec2::new(-0.5, 0.0))
        );
    }

    #[test]
    fn collision_rotate() {
        let a = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(1.0),
        };
        let ta = Transform2D {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        let b = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(1.0),
        };
        let tb = Transform2D {
            translation: Vec2::new(2.0,0.5),
            rotation: PI * 0.25,
            scale: Vec2::splat(1.0),
        };

        let a_b = a.collide(ta, &b, tb);
        let b_a = b.collide(tb, &a, ta);

        // make sure both see the collision...
        assert_eq!(a_b.is_some(), b_a.is_some());
        println!("{:?}", a_b);
        assert!((a_b.unwrap() + Vec2::new(2.0_f32.sqrt() - 1.0, 0.0)).length() < EPSILON);
    }

    #[test]
    fn collision_big() {
        let big = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::new(100.0,10.0),
        };

        let tbig = Transform2D {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        let a = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(10.0),
        };

        let ta = Transform2D {
            translation: Vec2::new(50.0, 10.0),
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        };

        // --------- TEST 1 -------------
        let big_a = big.collide(tbig, &a, ta);
        let a_bg = a.collide(ta, &big, tbig);

        // make sure both register
        assert_eq!(big_a.is_some(), a_bg.is_some());

        assert!((big_a.unwrap() - Vec2::new(0.0, -10.0)).length() < EPSILON);
        // this time imma check both for proper result
        assert!((a_bg.unwrap() - Vec2::new(0.0, 10.0)).length() < EPSILON);

        // ------- END TEST 1 ----------

        let r = Square {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
            extents: Vec2::splat(10.0),
        };

        let tr = Transform2D {
            translation: Vec2::new(-70.0,-10.0),
            rotation: 0.25 * PI,
            scale: Vec2::splat(1.0),
        };

        // ------ TEST 2 --------
        let big_r = big.collide(tbig, &r, tr);
        let r_big = r.collide(tr, &big, tbig);

        // both register
        assert_eq!(big_r.is_some(), r_big.is_some());

        assert!((big_r.unwrap() - Vec2::new(0.0, 10.0_f32 * 2.0_f32.sqrt())).length() < EPSILON);

    }
}
