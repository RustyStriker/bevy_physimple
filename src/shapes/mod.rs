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
    /// Returnts : segment.n * segment.n.dot(distance)
    fn get_segment_penetration(
        &self,
        segment : Segment,
        transform : Transform2D,
    ) -> f32;

    /// Check for a collision between 2 `Shape` objects at given `Transform2D`
    ///
    /// Should be used after `Aabb` checks with movement and everything
    fn collide_with_shape(
        &self,
        transform : Transform2D,
        shape : &dyn Shape,
        shape_trans : Transform2D,
    ) -> (Vec2, bool);
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
    /// Returns the minimum distance between the 2 segments projected on their normals and wether they collide
    fn collide(
        self,
        other : Segment,
    ) -> (Vec2, bool) {
        let np = self.n.perp();
        let c = (self.a + self.b) * 0.5;

        let ap = np.dot(self.a - c);
        let bp = np.dot(self.b - c);

        let oan = self.n.dot(other.a - c);
        let obn = self.b.dot(other.b - c);
        let oap = np.dot(other.a - c);
        let obp = np.dot(other.b - c);

        let coll_n = (oan >= 0.0 && obn <= 0.0) || (oan <= 0.0 && obn >= 0.0);
        // let coll_np = (oap <= bp && obp >= ap) || (oap )

        // FIXME
        (Vec2::ZERO, false)
    }
}
