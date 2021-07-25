use bevy::prelude::*;
use serde::{Serialize,Deserialize};


/// Object Bounding Volume
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Obv {
    pub offset : Vec2,
    pub shape : BoundingShape
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum BoundingShape {
	Aabb(Aabb),
	Circle(BoundingCircle),
}

/// Bounding circle
#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct BoundingCircle {
	pub radius : f32,
}

/// Axis aligned bounding box
#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Aabb {
    pub extents : Vec2,
}
impl Aabb {
    /// Creates a new AABB from extents(0.5 * absolute size)
    pub fn new(extents : Vec2) -> Aabb {
        Aabb {
            extents : extents.abs(),
        }
    }
    /// Creates a new AABB object from absolute size
    pub fn size(size : Vec2) -> Aabb {
        Aabb {
            extents : size.abs() * 0.5,
        }
    }
}