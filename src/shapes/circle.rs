use bevy::prelude::*;
use bevy::math::Mat2;
use serde::{Serialize,Deserialize};

use super::{Aabb, Transform2D, Shape};

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
	pub fn with_offset(mut self, offset : Vec2) -> Self {
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
    fn to_aabb(&self, transform : Transform2D) -> Aabb {
		// TODO Should we rotate the scale values as well?
        Aabb {
			extents : transform.scale * self.radius,
			position : transform.translation + self.offset,
		}
    }

    fn to_basis_aabb(&self, basis_inv : Mat2, transform : Transform2D) -> Aabb {
        Aabb {
			extents : basis_inv * transform.scale * self.radius,
			position : basis_inv * transform.translation + self.offset,
		}
    }

    fn to_aabb_move(&self, movement : Vec2, transform : Transform2D) -> Aabb {
		let prem = transform.translation + self.offset;
		let postm = prem + movement;

		let scaled_radius = Vec2::splat(self.radius) * transform.scale;

		let min = prem.min(postm) - scaled_radius; // Just subtract the radius from each element
		let max = prem.max(postm) + scaled_radius; // Just add the radius for each element

		let position = (min + max) / 2.0;
		let extents = (max - position).abs();

		Aabb {
		    extents,
		    position,
		}
	}

    fn to_basis_aabb_move(&self, basis_inv : Mat2, movement : Vec2, transform : Transform2D) -> Aabb {
		// Transform to the given basis
		let movement = basis_inv * movement;
        let pre = basis_inv * (transform.translation + self.offset);
		let post = pre + movement;

		let scaled_radius = Vec2::splat(self.radius) * (basis_inv * transform.scale);

		let min = pre.min(post) - scaled_radius;
		let max = pre.max(post) + scaled_radius;

		let position = (min + max) / 2.0;
		let extents = (max - position).abs();

		Aabb {
			extents,
			position,
		}
    }

    fn get_vertex_penetration(&self, vertex : Vec2, transform : Transform2D) -> Option<Vec2> {
        let vertex = vertex - (transform.translation + self.offset);

		// Shrink down the vertex based on scale
		let vertex = vertex * transform.scale.recip();

		let distance = vertex.length();

		if distance <= self.radius {
			let normal = vertex / distance; // Basically normalizing the vector

			Some(normal * (self.radius - distance))
		}
		else {
			None
		}
    }
}