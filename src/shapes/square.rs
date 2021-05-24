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
        let rot = Mat2::from_angle(transform.rotation);

		// We do the conjugate because if we have extents of (1,1) and we rotate 45deg we get (sqrt_2,0)
		let ex = (rot * (self.extents * transform.scale)).abs(); // we abs in case of a rotation more than 45 degrees
		let ex_con = (rot * (self.extents * Vec2::new(1.0,-1.0) * transform.scale)).abs();
		let extents = ex.max(ex_con);

		


		Aabb {
			extents,
			position : transform.translation + self.offset,
		}
    }

    fn to_basis_aabb(&self, basis_inv : Mat2, transform : Transform2D) -> Aabb {
		let position = basis_inv * (transform.translation + self.offset);
		
		let rot = Mat2::from_angle(transform.rotation);
		let ex = (rot * (basis_inv * (self.extents * transform.scale))).abs(); // we abs in case of a rotation more than 45 degrees
		let ex_con = (rot * (basis_inv * self.extents * Vec2::new(1.0,-1.0) * transform.scale)).abs();
		let extents = ex.max(ex_con);

		Aabb {
			extents,
			position,
		}
    }

    fn to_aabb_move(&self, movement : Vec2, transform : Transform2D) -> Aabb {
        let pre = transform.translation + self.offset;
		let post = pre + movement;

		let rot = Mat2::from_angle(transform.rotation);
		let ex = (rot * (self.extents * transform.scale)).abs(); // we abs in case of a rotation more than 45 degrees
		let ex_con = (rot * (self.extents * Vec2::new(1.0,-1.0) * transform.scale)).abs();
		let extents = ex.max(ex_con);

		let min = pre.min(post) - extents;
		let max = pre.max(post) + extents;

		let position = (min + max) * 0.5;
		let extents = (max - position).abs();

		Aabb {
			extents,
			position,
		}
    }

    fn to_basis_aabb_move(&self, basis_inv : Mat2, movement : Vec2, transform : Transform2D) -> Aabb {
		let pre = basis_inv * (transform.translation + self.offset);
		let post = pre + basis_inv * movement;

		let rot = Mat2::from_angle(transform.rotation);
		let ex = (rot * (basis_inv * (self.extents * transform.scale))).abs(); // we abs in case of a rotation more than 45 degrees
		let ex_con = (rot * (basis_inv * self.extents * Vec2::new(1.0,-1.0) * transform.scale)).abs();
		let extents = ex.max(ex_con);

		let min = pre.min(post) - extents;
		let max = pre.max(post) + extents;

		let position = (min + max) * 0.5;
		let extents = (max - position).abs();

		Aabb {
			extents,
			position,
		}
	}

    fn get_vertex_penetration(&self, vertex : Vec2, transform : Transform2D) -> (Vec2, bool) {
		let basis = Mat2::from_angle(transform.rotation);
		let basis_inv = basis.inverse();

		let vertex = vertex - (transform.translation + self.offset);
		let vertex = basis_inv * vertex * transform.scale.recip();

		let extents = self.extents;


		// Check that indeed it is between at least 2 parallel edges
		if vertex.abs() > extents {
			// It is not inside any pair of parallel edges
			// the closest edge will be the one where both signs of the edges will resemle the vertex's signs
			// so it will end up being `extents * vertex.signum()` therefore

			let distance = vertex - (vertex.signum() * extents);

			(distance, false)
		}
		else {
			// This is counter intuitive but you want the distance between `vertex.x` and `-extents.x`
			// 	 which in turns out as `vertex.x - (-extents.x) = vertex.x + extents.x`
			// let xmin = (vertex.x + extents.x).min(extents.x - vertex.x);
			// let ymin = (vertex.y + extents.y).min(extents.y - vertex.y);

			// let min = xmin.min(ymin);

			// // I will be honest, i dont really know why i need to do the `* vertex.[x/y].signum()` part
			// //  it just makes it work as expected(maybe it has something to do with the collision part of things?)
			// let res = if (min - xmin).abs() <= f32::EPSILON {
			// 	Vec2::new(xmin * vertex.x.signum(),0.0)
			// }
			// else {
			// 	Vec2::new(0.0,ymin * vertex.y.signum())
			// };

			let to_edge = extents.abs() - vertex.abs();
			let res = if to_edge.x < to_edge.y {
				Vec2::new(to_edge.x * vertex.x.signum(),0.0)
			}
			else {
				Vec2::new(0.0, to_edge.y * vertex.y.signum())
			};
			
			let is_pen = vertex.x.abs() < extents.x && vertex.y.abs() < extents.y;
			
			(basis * res, is_pen)
		}
	}

    fn collide_with_shape<S : Shape>(&self, transform : Transform2D, shape : &S, shape_trans : Transform2D) -> (Vec2, bool) {
		let rot_basis = Mat2::from_angle(transform.rotation);
		let extents = rot_basis * (self.extents * transform.scale);
		let extents_con = rot_basis * (self.extents * transform.scale * Vec2::new(1.0,-1.0));

        let center = transform.translation + self.offset;

		let vertices = [
			center + extents,
			center - extents,
			center + extents_con,
			center - extents_con,
		];

		let mut collide = false;
		let mut penetration = Vec2::splat(f32::INFINITY);

		for v in vertices.iter() {
			let (dis, is_pen) = shape.get_vertex_penetration(*v, shape_trans);
			
			if is_pen && !collide {
				penetration = Vec2::ZERO;
			}
			
			collide |= is_pen;
			if is_pen {
				if dis.length_squared() > penetration.length_squared() {
					penetration = dis;
				}
			}
			else if dis.length_squared() < penetration.length_squared() {
					penetration = dis;
			}
		}
		(penetration, collide)
    }
}

#[cfg(test)]
mod square_tests {
	use std::f32::consts::PI;

    use super::*;

	#[test]
	fn vertex_rotated_square() {
		let rect = Square::new(Vec2::new(10.0,5.0));

		let transform = Transform2D {
		    translation: Vec2::ZERO,
		    rotation: 0.5 * PI, // 90 degrees in radians... 
		    scale: Vec2::splat(1.0),
		};

		let outside = Vec2::new(7.0,7.0);
		println!("vertex : {:?}",outside);

		let (pen, colliding) = rect.get_vertex_penetration(outside, transform);

		let res = (pen - Vec2::new(-2.0,0.0)).abs();
		
		// Use a much higher value of epsilon due to the trigo functions in the rotation calculations having
		//  around 0.0000005 miss
		const EPSILON : f32 = 0.001;
		eprintln!("res {:?}, pen {:?}",res, pen);

		assert!(res.x <= EPSILON && res.y <= EPSILON && !colliding);

	}

	#[test]
	fn vertex_inside_square() {
		let rect = Square::new(Vec2::new(10.0,5.0));

		let transform = Transform2D {
		    translation: Vec2::ZERO,
		    rotation: 0.5 * PI,
		    scale: Vec2::splat(1.0),
		};

		let vertex = Vec2::new(-3.0,7.0);
		
		let (pen, coll) = rect.get_vertex_penetration(vertex, transform);

		let res = (pen - Vec2::new(-2.0,0.0)).abs();

		const EPSILON : f32 = 0.001;
		eprintln!("res {:?} pen {:?}",res,pen);

		assert!(res.x <= EPSILON && res.y <= EPSILON && coll);
	}
}