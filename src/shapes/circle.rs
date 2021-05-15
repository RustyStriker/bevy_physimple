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
		// rotate the scale 
		let basis = Mat2::from_angle(transform.rotation);
		let scale = basis * transform.scale;

        Aabb {
			extents : scale * self.radius,
			position : transform.translation + self.offset,
		}
    }

    fn to_basis_aabb(&self, basis_inv : Mat2, transform : Transform2D) -> Aabb {
		// rotate the scale
		let rot_basis = Mat2::from_angle(transform.rotation);
		let scale = rot_basis * transform.scale;

        Aabb {
			extents : basis_inv * scale * self.radius,
			position : basis_inv * transform.translation + self.offset,
		}
    }

    fn to_aabb_move(&self, movement : Vec2, transform : Transform2D) -> Aabb {
		let prem = transform.translation + self.offset;
		let postm = prem + movement;

		let rot_basis = Mat2::from_angle(transform.rotation);
		let scale = rot_basis * transform.scale;

		let scaled_radius = Vec2::splat(self.radius) * scale;

		let min = prem.min(postm) - scaled_radius; // Just subtract the radius from each element
		let max = prem.max(postm) + scaled_radius; // Just add the radius for each element

		let position = (min + max) * 0.5;
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

		let rot_basis = Mat2::from_angle(transform.rotation);
		let scale = rot_basis * transform.scale;

		let scaled_radius = Vec2::splat(self.radius) * (basis_inv * scale);

		let min = pre.min(post) - scaled_radius;
		let max = pre.max(post) + scaled_radius;

		let position = (min + max) * 0.5;
		let extents = (max - position).abs();

		Aabb {
			extents,
			position,
		}
    }

    fn get_vertex_penetration(&self, vertex : Vec2, transform : Transform2D) -> (Vec2, bool) {
        let vertex = vertex - (transform.translation + self.offset);

		// Shrink down the vertex based on scale
		let vertex = vertex * transform.scale.recip();

		let distance = vertex.length();


		let normal = vertex / distance; // Basically normalizing the vector

		(normal * (self.radius - distance), distance < self.radius) // Return the penetration value

    }

    fn collide_with_shape<S : Shape>(&self, transform : Transform2D, shape : &S, shape_trans : Transform2D) -> (Vec2, bool) {
        let center = transform.translation + self.offset;

		let (dis, is_pen) = shape.get_vertex_penetration(center, shape_trans);

		if is_pen {
			let normal = dis.normalize();
			let pen = dis + normal * self.radius;

			(pen, true)
		}
		else {
			let dis_len = dis.length();
			// calculate the distance to the shape
			let pen = (self.radius - dis.length()) * dis / dis.length();
			if dis_len < self.radius {
				(-pen, true)
			}
			else {
				(pen, false)
			}
		}
    }
}

#[cfg(test)]
mod circle_tests {
	use super::*;

	#[test]
	fn vertex_testing() {
		let circle = Circle::new(5.0)
			.with_offset(Vec2::new(5.0,0.0));
		
		let transform = Transform2D {
		    translation: Vec2::ZERO,
		    rotation: 0.0,
		    scale: Vec2::splat(1.0),	
		};

		let vertex_a = Vec2::new(0.0,5.0); // Shouldnt be inside

		let (_, a_coll) = circle.get_vertex_penetration(vertex_a, transform);
		// vertex shouldnt be inside the thing
		assert!(!a_coll);

		let vertex_b = Vec2::new(2.0,0.0); // should be inside

		let (b_pen, b_coll) = circle.get_vertex_penetration(vertex_b, transform);

		assert!(b_coll);
		assert_eq!(b_pen, Vec2::new(-2.0,0.0));
	}
}