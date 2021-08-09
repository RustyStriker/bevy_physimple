use crate::settings::TransformMode;
use bevy::prelude::*;

mod circle;
mod raycast;
mod square;
mod obv;
mod segment;

pub use circle::*;
pub use raycast::*;
pub use square::*;
pub use obv::*;
pub use segment::Segment;

pub trait Shape {
    /// Returns an Aabb instance containing the shape
    fn to_aabb(
        &self,
        transform : Transform2D,
    ) -> Aabb;
    /// Returns the distance(as `Vec2`) from the shape to the vertex
    ///
    /// Returns : (distance from edge, is_penetrating)
    fn collide_vertex(
        &self,
        vertex : Vec2,
        transform : Transform2D,
    ) -> (Vec2, bool);

    /// Returns the distance(as result of `normal * normal.dot(distance)`) from the shape to the segment
    ///
    /// Returnts : segment.n.dot(distance)
    fn collide_segment(
        &self,
        segment : Segment,
        transform : Transform2D,
        segment_origin : Vec2,
    ) -> f32;

    /// Check for a collision between 2 `Shape` objects at given `Transform2D`
    /// And returns the penetration.
    ///
    /// In order to solve the collision: `translation = translation - collide(...).unwrap()`
    ///
    /// Should be used after `Aabb` checks with movement and everything
    fn collide(
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

pub enum CollisionShape {
    Circle(Circle),
    Square(Square),
}
impl CollisionShape {
    /// Returns a fat reference to the actual shape(based on the `Shape` trait)
    pub fn shape(&self) -> &dyn Shape {
        match self {
            CollisionShape::Circle(c) => c,
            CollisionShape::Square(s) => s,
        }
    }
}