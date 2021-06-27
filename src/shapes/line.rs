use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Aabb, Shape, Transform2D};

/// 2D Line represented as 2 points(relative to the connected `GlobalTransform` component)
#[derive(Clone, Serialize, Deserialize, Debug, Reflect)]
pub struct Line2D {
    pub a: Vec2,
    pub b: Vec2,
}
impl Line2D {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Line2D { a, b }
    }
}
impl Shape for Line2D {
    fn to_aabb(&self, transform: Transform2D) -> Aabb {
        let basis = Mat2::from_angle(transform.rotation);
        // rotate the points
        let a = basis * self.a;
        let b = basis * self.b;

        let min = a.min(b) + transform.translation;
        let max = a.max(b) + transform.translation;

        let position = (min + max) * 0.5;
        let extents = (max - position).abs() * transform.scale;

        Aabb { extents, position }
    }

    fn to_basis_aabb(&self, basis_inv: Mat2, transform: Transform2D) -> Aabb {
        // Convert stuff to local coordinates
        let a = basis_inv * self.a;
        let b = basis_inv * self.b;
        let scale = basis_inv * transform.scale;
        let position = basis_inv * transform.translation;

        // rotate the points
        let rotation_basis = Mat2::from_angle(transform.rotation);
        let a = rotation_basis * a;
        let b = rotation_basis * b;

        let min = a.min(b) + position;
        let max = a.max(b) + position;

        let position = (min + max) * 0.5;
        let extents = (max - position).abs() * scale;

        Aabb { extents, position }
    }

    fn to_aabb_move(&self, movement: Vec2, transform: Transform2D) -> Aabb {
        // get the points to be rotated and such
        let rot_basis = Mat2::from_angle(transform.rotation);

        let a = rot_basis * self.a;
        let b = rot_basis * self.b;

        // lock them in an aabb(or extents at least)
        let min = a.min(b);
        let max = a.max(b);

        let offset = (min + max) * 0.5;
        let extents = (max - offset).abs() * transform.scale;

        let pre = transform.translation + offset;
        let post = pre + movement;

        // recalculate the min/max for the containing box
        let min = pre.min(post) - extents;
        let max = pre.max(post) + extents;

        let position = (min + max) * 0.5;
        let extents = (max - position).abs();

        Aabb { extents, position }
    }

    fn to_basis_aabb_move(
        &self,
        _basis_inv: Mat2,
        _movement: Vec2,
        _transform: Transform2D,
    ) -> Aabb {
        todo!()
    }

    fn get_vertex_penetration(&self, _vertex: Vec2, _transform: Transform2D) -> (Vec2, bool) {
        todo!()
    }

    fn collide_with_shape(
        &self,
        _transform: Transform2D,
        _shape: &dyn Shape,
        _shape_trans: Transform2D,
    ) -> (Vec2, bool) {
        todo!()
    }
}
