use std::slice::{Iter, IterMut};

use bevy::prelude::*;
use serde::{Deserialize,Serialize};

#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct Sensor2D {
	/// Which collision layers this body search collisions for
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub mask : u8,
	/// Which collision layers this body occupies
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub layer : u8,

	pub(crate) overlapping_bodies : Vec<Entity>,
}

impl Sensor2D {
	pub fn new() -> Self {
		Sensor2D {
			mask : 1,
			layer : 1,
			overlapping_bodies : Vec::with_capacity(5),
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

	pub fn iter_overlapping_bodies(&self) -> Iter<'_, Entity> {
		self.overlapping_bodies.iter()
	}
	pub fn iter_overlapping_bodies_mut(&mut self) -> IterMut<'_,Entity> {
		self.overlapping_bodies.iter_mut()
	}
}
impl Default for Sensor2D {
    fn default() -> Self {
        Self::new()
    }
}