use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// StaticBody for 2D physics(with supposedly infinite mass)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StaticBody2D {
	/// Current position of the static body
	pub position : Vec2,
	/// Current rotation of the static body
	pub rotation : f32,

	/// Which collision layers this body search collisions for
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub mask : u8,
	/// Which collision layers this body occupies
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub layer : u8,

	// TODO Add StaticBody bounciness
	// Basically how trampoline like the object is
}
impl StaticBody2D {
	/// Creates a new StaticBody with default parameters
	pub fn new() -> Self {
		Self {
			position : Vec2::ZERO,
			rotation : 0.0,
			mask : 1,
			layer : 1,
		}
	}
	pub fn with_position(mut self, position : Vec2) -> Self {
		self.position = position;
		self
	}
	pub fn with_rotation(mut self, rotation : f32) -> Self {
		self.rotation = rotation;
		self
	}
	pub fn with_mask(mut self, mask : u8) -> Self {
		self.mask = mask;
		self
	}
	pub fn with_layer(mut self, layer : u8) -> Self {
		self.layer = layer;
		self
	}
}