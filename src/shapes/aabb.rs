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

    pub fn min_max(&self) -> (Vec2,Vec2) {
        (self.position - self.extents, self.position + self.extents)
    }

    pub fn collides(&self, other: &Aabb) -> bool {
        let (min1, max1) = self.min_max();
        let (min2, max2) = other.min_max();

        min1 < max2 && min2 < max1
    }
}
