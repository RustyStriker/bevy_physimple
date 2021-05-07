use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// StaticBody for 2D physics(with supposedly infinite mass)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StaticBody2D {
	/// Which collision layers this body search collisions for
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub mask : u8,
	/// Which collision layers this body occupies
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub layer : u8,

	/// Basically how trampoline like the object is (default - 0)
	///
	/// (0 - hard, 1 - full trampoline, >1 funny and weird)
	pub bounciness : f32,

}
impl StaticBody2D {
	/// Creates a new StaticBody with default parameters
	pub fn new() -> Self {
		Self {
			// position : Vec2::ZERO,
			// rotation : 0.0,
			mask : 1,
			layer : 1,
			bounciness : 0.0,
		}
	}
	pub fn with_mask(mut self, mask : u8) -> Self {
		self.mask = mask;
		self
	}
	pub fn with_layer(mut self, layer : u8) -> Self {
		self.layer = layer;
		self
	}
	pub fn with_bounciness(mut self, bounce : f32) -> Self {
		self.bounciness = bounce;
		self
	}
}
impl Default for StaticBody2D {
	fn default() -> Self {
		Self::new()
	}
}