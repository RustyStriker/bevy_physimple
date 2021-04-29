use bevy::prelude::*;
use serde::{Serialize, Deserialize};

pub trait ToAABB {
	fn aabb(&self) -> AABB;
}

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct AABB {
	pub(crate) extents : Vec2
}
impl AABB {
	/// Creates a new AABB from extents(0.5 * absolute size)
	pub fn new(extents : Vec2) -> AABB {
		AABB {
			extents : extents.abs()
		}
	}
	/// Creates a new AABB object from absolute size
	pub fn size(size : Vec2) -> AABB {
		AABB {
			extents : size.abs() * 0.5
		}
	}
}
impl ToAABB for AABB {
	fn aabb(&self) -> AABB {
		*self
	}
}

/// Temp struct for aabb collision event
#[derive(Clone, Debug)]
pub struct AABBCollisionEvent {
    pub entity_a : Entity,
    pub entity_b : Entity,
    /// Penetration of a in b, can get normal out of it
    pub penetration : Vec2,
    /// If the collision happened with a static body
    pub with_static : bool
}