use crate::physics_components::Transform2D;
use bevy::prelude::*;

mod aabb;
mod circle;
mod raycast;
mod segment;
mod square;

pub use aabb::*;
pub use circle::*;
pub use raycast::*;
pub use segment::Segment;
pub use square::*;

pub trait Shape {
    /// Returns an Aabb instance containing the shape
    fn to_aabb(
        &self,
        transform: &Transform2D,
    ) -> Aabb;
    /// Returns the distance(as `Vec2`) from the shape to the vertex
    ///
    /// Returns : (distance from edge, is_penetrating)
    fn collide_vertex(
        &self,
        vertex: Vec2,
        transform: &Transform2D,
    ) -> (Vec2, bool);

    /// Returns the distance(as result of `normal * normal.dot(distance)`) from the shape to the segment
    ///
    /// Returnts : segment.n.dot(distance)
    fn collide_segment(
        &self,
        segment: Segment,
        transform: &Transform2D,
    ) -> f32;

    /// Check for a collision between 2 `Shape` objects at given `Transform2D`
    /// And returns the penetration.
    ///
    /// Should be used after `Aabb` checks with movement and everything
    fn collide(
        &self,
        transform: &Transform2D,
        shape: &dyn Shape,
        shape_trans: &Transform2D,
    ) -> Option<Vec2>;

    /// Returns collision with the ray
    ///
    /// ray = (ray_normal, ray_length)
    fn collide_ray(
        &self,
        transform: &Transform2D,
        ray: (Vec2, f32),
        ray_origin: Vec2,
    ) -> Option<f32>;
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
