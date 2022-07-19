use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Axis aligned bounding box
#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Aabb {
    pub extents: Vec2,
    pub position: Vec2,
}
impl Aabb {
    /// Creates a new AABB from extents(0.5 * absolute size)
    pub fn new(extents: Vec2, position: Vec2) -> Aabb {
        Aabb {
            extents: extents.abs(),
            position
        }
    }
    /// Creates a new AABB object from absolute size
    pub fn size(size: Vec2, position: Vec2) -> Aabb {
        Aabb {
            extents: size.abs() * 0.5,
            position,
        }
    }
    /// Creates a new AABB object from a minimum and maximum extents
    pub fn from_min_max(min: Vec2, max: Vec2) -> Aabb {
        Aabb::size(max - min, (min + max) * 0.5)
    }

    pub fn min_max(&self) -> (Vec2,Vec2) {
        (self.position - self.extents, self.position + self.extents)
    }

    pub fn collides(&self, other: &Aabb) -> bool {
        let (min1, max1) = self.min_max();
        let (min2, max2) = other.min_max();

        min1.x < max2.x && min1.y < max2.y &&
        min2.x < max1.x && min2.y < max1.y
    }
}
