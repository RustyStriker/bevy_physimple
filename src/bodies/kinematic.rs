use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// KinematicBody for 2D physics, for moving objects
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct KinematicBody2D {
	/// Current position
	pub position : Vec2,
	/// Rotation in radians
	pub rotation : f32,

	/// Linear velocity
	pub linvel : Vec2,

	/// Terminal linear velocity
	///
	/// Defaults to `f32::INFINITY`
	pub terminal : Vec2,
	
	pub(crate) accumulator : Vec2,
	pub(crate) dynamic_acc : Vec2,

	/// Angular velocity
	pub angvel : f32,
	/// Terminal angular velocity
	pub ang_terminal : f32,

	pub(crate) mass : f32,
	pub(crate) inv_mass : f32,

	/// Which collision layers this body search collisions for
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub mask : u8,
	/// Which collision layers this body occupies
	///
	/// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
	pub layer : u8,

	/// Whether the body will participate in all physics related systems
	pub active : bool,

	/// Some(normal) if on_floor
	pub(crate) on_floor : Option<Vec2>,
	/// Some(normal) if on_wall
	pub(crate) on_wall : Option<Vec2>,
	/// Some(normal) if on_ceil
	pub(crate) on_ceil : Option<Vec2>,

	// TODO Add bounciness factor
}

impl KinematicBody2D {
	/// Returns a new 'default' KinematicBody2D
	pub fn new() -> Self {
		KinematicBody2D {
		    position: Vec2::ZERO,
		    rotation: 0.0,
		    linvel: Vec2::ZERO,
		    terminal: Vec2::new(f32::INFINITY,f32::INFINITY),
		    accumulator: Vec2::ZERO,
		    dynamic_acc: Vec2::ZERO,
		    angvel: 0.0,
		    ang_terminal: f32::INFINITY,
		    mass: 1.0,
		    inv_mass: 1.0, // inverse of 1.0 O-O
		    mask: 1,
		    layer: 1,
			active : false,
		    on_floor: None,
		    on_wall: None,
		    on_ceil: None,
		}
	}
	pub fn with_position(mut self, position : Vec2) -> Self {
		self.position = position;
		self
	}
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
    pub fn with_linear_velocity(mut self, linvel: Vec2) -> Self {
        self.linvel = linvel;
        self
    }
    pub fn with_angular_velocity(mut self, angvel: f32) -> Self {
        self.angvel = angvel;
        self
    }
    pub fn with_terminal(mut self, terminal: Vec2) -> Self {
        self.terminal = terminal;
        self
    }
    pub fn with_angular_terminal(mut self, terminal: f32) -> Self {
        self.ang_terminal = terminal;
        self
    }
	pub fn with_mass(mut self, mass : f32) -> Self {
		self.mass = mass;
		self.inv_mass = mass.recip();
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
	pub fn with_active(mut self, active : bool) -> Self {
		self.active = active;
		self
	}
    
	/// Apply an impulse to the body
	///
	/// does not handle with delta_time
	pub fn apply_linear_impulse(&mut self, impulse : Vec2) {
		self.linvel += impulse * self.inv_mass;
	}
	/// Apply an angular impulse to the body
	///
	/// does not handle with delta_time
	pub fn apply_angular_impulse(&mut self, impulse : f32) {
		self.angvel += impulse * self.inv_mass;
	}
	/// Applies a force
	///
	/// handles with delta_time
	pub fn apply_force(&mut self, force : Vec2) {
		self.accumulator += force * self.inv_mass
	}
	/// Gets the mass
	pub fn mass(&self) -> f32 {
		self.mass
	}
	/// Gets the inverse mass
	pub fn inverse_mass(&self) -> f32 {
		self.inv_mass
	}
	/// Sets the mass
	pub fn set_mass(&mut self, mass : f32) {
		self.mass = mass;
		self.inv_mass = mass.recip();
	}
	/// Get Floor normal if body is on floor
    pub fn on_floor(&self) -> Option<Vec2> {
        self.on_floor
    }
    /// Get wall normal if body is touching a wall
    pub fn on_wall(&self) -> Option<Vec2> {
        self.on_wall
    }
    /// Get ceilling normal if body is touching a ceiling
    pub fn on_ceil(&self) -> Option<Vec2> {
        self.on_ceil
    }
}