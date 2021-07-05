use crate::plugin::TransformMode;
use bevy::prelude::*;

mod aabb;
mod circle;
mod raycast;
mod square;

pub use aabb::*;
pub use circle::*;
pub use raycast::*;
pub use square::*;

pub trait Shape {
    /// Returns an Aabb instance containing the shape
    fn to_aabb(
        &self,
        transform : Transform2D,
    ) -> Aabb;
    /// Returns the distance(as `Vec2`) from the shape to the vertex
    ///
    /// Returns : (distance from edge, is_penetrating)
    fn get_vertex_penetration(
        &self,
        vertex : Vec2,
        transform : Transform2D,
    ) -> (Vec2, bool);

    /// Returns the distance(as result of `normal * normal.dot(distance)`) from the shape to the segment
    ///
    /// Returnts : segment.n.dot(distance)
    fn get_segment_penetration(
        &self,
        segment : Segment,
        transform : Transform2D,
        segment_origin : Vec2,
    ) -> f32;

    /// Check for a collision between 2 `Shape` objects at given `Transform2D`
    ///
    /// Should be used after `Aabb` checks with movement and everything
    fn collide_with_shape(
        &self,
        transform : Transform2D,
        shape : &dyn Shape,
        shape_trans : Transform2D,
    ) -> Option<Vec2>;
}

/// This is a temporary struct until bevy gets it own `Transform2D` struct
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Transform2D {
    pub translation : Vec2,
    pub rotation : f32,
    pub scale : Vec2,
}
impl From<(&GlobalTransform, TransformMode)> for Transform2D {
    fn from((trans, mode) : (&GlobalTransform, TransformMode)) -> Self {
        let t = trans.translation;
        let r = trans.rotation;
        let s = trans.scale;

        match mode {
            TransformMode::XY => Transform2D {
                translation : Vec2::new(t.x, t.y),
                rotation : r.z,
                scale : Vec2::new(s.x, s.y),
            },
            TransformMode::XZ => Transform2D {
                translation : Vec2::new(t.x, t.z),
                rotation : r.y,
                scale : Vec2::new(s.x, s.z),
            },
            TransformMode::YZ => Transform2D {
                translation : Vec2::new(t.y, t.z),
                rotation : r.x,
                scale : Vec2::new(s.y, s.z),
            },
        }
    }
}

/// Simple struct to represent a segment from a to b
#[derive(Clone, Copy, Reflect, Debug)]
pub struct Segment {
    /// Point a
    pub a : Vec2,
    /// Point b
    pub b : Vec2,
    /// Normal
    pub n : Vec2,
}
impl Segment {
    /// Returns the `a` where `penetration = a * self.normal`
    ///
    /// if `a > 0.0` -> no penetration happend, this is the distance
    fn collide(
        self,
        other : Segment,
    ) -> Option<f32> {
        let np = self.n.perp();
        let c = (self.a + self.b) * 0.5;

        let ap = np.dot(self.a - c);
        let bp = np.dot(self.b - c);

        let oap = np.dot(other.a - c);
        let obp = np.dot(other.b - c);

        let np_min = ap.min(bp);
        let np_max = ap.max(bp);

        let op_min = oap.min(obp);
        let op_max = oap.max(obp);

        if op_min <= np_max && op_max >= np_min {
            let oan = self.n.dot(other.a - c);
            let obn = self.n.dot(other.b - c);

            let min = oan.min(obn);

            Some(min)
        }
        else {
            None
        }
    }

    /// Returns the minimum distance between the segment and a given point
    ///
    /// Returns: (length on normal, length perp to normal)
    fn collide_point(
        self,
        point : Vec2,
    ) -> (f32, f32) {
        let np = self.n.perp();
        let c = (self.a + self.b) * 0.5;

        let ap = np.dot(self.a - c);
        let bp = np.dot(self.b - c);

        let pp = np.dot(point - c);

        let np_part = if pp >= ap.min(bp) && pp <= ap.max(bp) {
            0.0
        }
        else {
            let a = pp - ap;
            let b = pp - bp;
            if a.abs() > b.abs() {
                a
            }
            else {
                b
            }
        };

        (self.n.dot(point - c), np_part)
    }
}
