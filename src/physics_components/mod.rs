//! All the different components which describe a physical body

mod velocity;
mod transform2d;
pub use transform2d::Transform2D;
pub use velocity::Vel;

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

/**
    # CollisionLayer

    Holds both the `layer` and `mask` of the entity.

    The mask field sets what collision layers the object lays in,
    
    The layer field sets what collision layesr the boject will check for in collision,
    
    Both fields are represented as the individual bits in a `u8`(so there are 8 layers).

    A Collision can occur between 2 objects(`a` and `b` are their `CollisionLayer`s) only when `(a.mask & b.layer) | (a.layer & b.mask) != 0`,
    or a.overlap(b) for short.

    ## Adding/Removing Layers(applies for masks as well)

    The easiest way to handle layers is to flip them using he `^`(xor - exclusive or) operator,
    we can flip a specific layer(for example, the forth layer) by doing `layer = layer ^ 0x0000_1000`.

    Flipping a layer will add it if it wasn't added already, and remove it if it was added before.

    Adding a layer(for example, the third layer) is as simple as doing `layer = layer | 0x0000_0100...`.

    Removing a specific layer(without flipping it) is rather a problem,
    we will need to use the `&` operator, but for each bit we didnt write,
    the compiler will assume it as `0`,
    but since we are working with `u8` we can simply write all the bits,
    so to remove a layer(for example, the second layer) we will do `layer = layer & 0x1111_1101`.

    We can also add/remove/flip multiple layers at a time.

    For example, if we want to add layers 2 and 3 in one go, we can do `layer = layer | 0x000_0110`
*/
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct CollisionLayer {
    pub mask: u8,
    pub layer: u8,
}

impl Default for CollisionLayer {
    fn default() -> Self {
        Self { mask: 1, layer: 1 }
    }
}
impl CollisionLayer {
    /// CollisionLayer without any layer/mask activated
    pub const ZERO: CollisionLayer = CollisionLayer { mask: 0, layer: 0};

    pub fn new(
        mask: u8,
        layer: u8,
    ) -> Self {
        Self { mask, layer }
    }
    /// Checks if 2 `CollisionLayer`s should collide with each other
    pub fn overlap(
        &self,
        other: &CollisionLayer,
    ) -> bool {
        (self.mask & other.layer) | (self.layer & other.mask) != 0
    }
}

