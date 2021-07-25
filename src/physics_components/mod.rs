//! All the different components which describe a physical body

/*
	# Components needed:
		[x] Velocity
		[x] Angular Velocity
		[x] Terminal (angualr) Velocity
		[x] Mass
		[x] Bounding volume(currently should be circle/box)
		[x] Collision Mask
		[x] Collision Layers(Maybe should be the same with mask)
		[ ] bounce
		[ ] stiffness
		[x] Friction

	still TODODODODODO:
		[ ] Update all systems to work with new components
		[ ] implement quality of life functions for new component
		[ ] create builders/bundles for the PhysicsBodies :D
*/



pub mod velocity;
pub mod angular_velocity;
pub mod physical_properties;

use bevy::{prelude::Reflect};
use serde::{Serialize,Deserialize};

#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct CollisionLayer {
	pub mask : u8,
	pub layer : u8,
}
impl CollisionLayer {
	pub fn new(mask : u8, layer : u8) -> Self {
		Self{
			mask,
			layer
		}
	}
	pub fn overlap(&self, other : &CollisionLayer) -> bool {
		(self.mask & other.layer) | (self.layer & other.mask) != 0 
	}
}
