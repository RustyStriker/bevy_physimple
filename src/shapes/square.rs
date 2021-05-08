use bevy::prelude::*;
use bevy::math::Mat2;
use serde::{Serialize,Deserialize};

use super::{Aabb, Transform2D, Shape};

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
			extents : size,
		}
	}
	/// Offset from the `Transform` transltion component
	pub fn with_offset(mut self, offset : Vec2) -> Self {
		self.offset = offset;
		self
	}
	/// rotation offset from the `Transform` rotation component
	pub fn with_rotation_offset(mut self, offset : f32) -> Self {
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
    fn to_aabb(&self, transform : Transform2D) -> Aabb {
        // TODO take care of rotations
		let ex = self.extents;
		let sc = transform.scale;

		Aabb {
			extents : ex * sc,
			position : transform.translation + self.offset,
		}
    }

    fn to_basis_aabb(&self, basis_inv : Mat2, transform : Transform2D) -> Aabb {
		let scale = basis_inv * transform.scale;

        let extents = (basis_inv * self.extents * scale).abs();
		let position = basis_inv * (transform.translation + self.offset);

		Aabb {
			extents,
			position,
		}
    }

    fn to_aabb_move(&self, movement : Vec2, transform : Transform2D) -> Aabb {
        let pre = transform.translation + self.offset;
		let post = pre + movement;

		let min = pre.min(post) - self.extents * transform.scale;
		let max = pre.max(post) + self.extents * transform.scale;

		let position = (min + max) / 2.0;
		let extents = (max - position).abs();

		Aabb {
			extents,
			position,
		}
    }

    fn to_basis_aabb_move(&self, basis : Mat2, movement : Vec2, transform : Transform2D) -> Aabb {
        todo!()
    }

    fn get_vertex_penetration(&self, vertex : Vec2, transform : Transform2D) -> Option<Vec2> {
		todo!()
	}
}