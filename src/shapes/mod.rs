use bevy::{math::Mat2, prelude::*};
use crate::plugin::TransformMode;

mod aabb;
mod raycast;
mod square;
mod circle;
mod line;

pub use aabb::*;
pub use raycast::*;
pub use square::*;
pub use circle::*;
pub use line::*;

pub trait Shape {
	/// Returns an Aabb instance containing the shape
	fn to_aabb(&self, transform : Transform2D) -> Aabb;
	/// Returns an Aabb instance on the basis given axis containing the shape
	fn to_basis_aabb(&self, basis_inv : Mat2, transform : Transform2D) -> Aabb;
	/// Returns an Aabb isntance containing the shape both before and after movement
	fn to_aabb_move(&self, movement : Vec2, transform : Transform2D) -> Aabb;
	/// returns an Aabb instance on the basis given axis containing the shape before and after movement
	fn to_basis_aabb_move(&self, basis_inv : Mat2, movement : Vec2, transform : Transform2D) -> Aabb;
	/// Returns the distance(as `Vec2`) from the shape to the vertex
	///
	/// Returns : (distance from edge, is_penetrating)
	fn get_vertex_penetration(&self, vertex : Vec2, transform : Transform2D) -> (Vec2, bool);

	fn collide_with_shape<S : Shape>(&self, transform : Transform2D, shape : &S, shape_trans : Transform2D) -> (Vec2, bool);
}

/// This is a temporary struct until bevy gets it own `Transform2D` struct
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Transform2D {
	pub translation : Vec2,
	pub rotation : f32,
	pub scale : Vec2,
}
impl From<(GlobalTransform, TransformMode)> for Transform2D {
    fn from((trans, mode): (GlobalTransform, TransformMode)) -> Self {
		let t = trans.translation;
		let r = trans.rotation;
		let s = trans.scale;

        match mode {
            TransformMode::XY => Transform2D {
				translation : Vec2::new(t.x,t.y),
				rotation : r.z,
				scale : Vec2::new(s.x,s.y),
			},
            TransformMode::XZ => Transform2D {
				translation : Vec2::new(t.x,t.z),
				rotation : r.y,
				scale : Vec2::new(s.x,s.z),
			},
            TransformMode::YZ => Transform2D {
				translation : Vec2::new(t.y,t.z),
				rotation : r.x,
				scale : Vec2::new(s.y,s.z),
			},
        }
    }
}