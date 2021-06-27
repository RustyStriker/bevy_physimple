use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// KinematicBody for 2D physics, for moving objects
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct KinematicBody2D {
    /// Linear velocity
    pub linvel: Vec2,
    /// Intentansious linvel of a single frame
    pub(crate) inst_linvel: Vec2,

    /// Terminal linear velocity
    ///
    /// Defaults to `f32::INFINITY`
    pub terminal: Vec2,

    pub(crate) accumulator: Vec2,
    pub(crate) dynamic_acc: Vec2,

    /// Angular velocity
    pub angvel: f32,
    /// Terminal angular velocity
    pub ang_terminal: f32,

    pub(crate) mass: f32,
    pub(crate) inv_mass: f32,

    /// Which collision layers this body search collisions for
    ///
    /// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
    pub mask: u8,
    /// Which collision layers this body occupies
    ///
    /// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
    pub layer: u8,

    /// Whether the body will participate in all physics related systems
    pub active: bool,

    /// How stiff the object is(as of "how much energy will transfer to heat on collision")
    ///
    /// 1(default) - no "heat", 0 - "all to heat"
    pub stiffness: f32,
    /// How bouncy a body is on collisions with static bodies
    ///
    /// 0(default) - no bounce, 1 - full bounce, >1 - super bouncy
    ///
    /// Used as `staticbody.bounciness.max(self.bounciness)` to act as bounciness override
    pub bounciness: f32,

    /// Friction is a constant force slowing the body down(default - 1.0)
    ///
    /// The constant global friction will be multiplied by this variable when calculated
    pub friction_mult: f32,

    /// Some(normal) if on_floor
    pub(crate) on_floor: Option<Vec2>,
    /// Some(normal) if on_wall
    pub(crate) on_wall: Option<Vec2>,
    /// Some(normal) if on_ceil
    pub(crate) on_ceil: Option<Vec2>,
}

impl KinematicBody2D {
    /// Returns a new 'default' KinematicBody2D
    pub fn new() -> Self {
        KinematicBody2D {
            linvel: Vec2::ZERO,
            inst_linvel: Vec2::ZERO,
            terminal: Vec2::new(f32::INFINITY, f32::INFINITY),
            accumulator: Vec2::ZERO,
            dynamic_acc: Vec2::ZERO,
            angvel: 0.0,
            ang_terminal: f32::INFINITY,
            mass: 1.0,
            inv_mass: 1.0, // inverse of 1.0 O-O
            mask: 1,
            layer: 1,
            active: true,
            stiffness: 1.0,
            bounciness: 0.0,
            friction_mult: 1.0,
            on_floor: None,
            on_wall: None,
            on_ceil: None,
        }
    }
    /// Linear velocity
    pub fn with_linear_velocity(mut self, linvel: Vec2) -> Self {
        self.linvel = linvel;
        self
    }
    /// Angular velocity
    pub fn with_angular_velocity(mut self, angvel: f32) -> Self {
        self.angvel = angvel;
        self
    }
    /// Terminal linear velocity
    ///
    /// Defaults to `f32::INFINITY`
    pub fn with_terminal(mut self, terminal: Vec2) -> Self {
        self.terminal = terminal;
        self
    }
    /// Terminal angular velocity
    pub fn with_angular_terminal(mut self, terminal: f32) -> Self {
        self.ang_terminal = terminal;
        self
    }
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self.inv_mass = mass.recip();
        self
    }
    /// Which collision layers this body search collisions for
    ///
    /// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
    pub fn with_mask(mut self, mask: u8) -> Self {
        self.mask = mask;
        self
    }
    /// Which collision layers this body occupies
    ///
    /// Generally to bodies will colide if (a.mask & b.layer) | (b.mask & a.layer) > 0
    pub fn with_layer(mut self, layer: u8) -> Self {
        self.layer = layer;
        self
    }
    /// Whether the body will participate in all physics related systems
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    /// How stiff the body is(as of "how much energy will transfer to heat on collision")
    ///
    /// 1(default) - no "heat", 0 - "all to heat"
    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }
    /// How bouncy a body is on collisions with static bodies
    ///
    /// 0(default) - no bounce, 1 - full bounce, >1 - super bouncy
    ///
    /// Used as `staticbody.bounciness.max(self.bounciness)` to act as bounciness override
    pub fn with_bounciness(mut self, bounciness: f32) -> Self {
        self.bounciness = bounciness;
        self
    }
    /// Friction is a constant force slowing the body down(default - 1.0)
    ///
    /// The constant global friction will be multiplied by this variable when calculated
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction_mult = friction;
        self
    }

    /// Apply an impulse to the body
    ///
    /// does not handle with delta_time
    pub fn apply_linear_impulse(&mut self, impulse: Vec2) {
        self.linvel += impulse * self.inv_mass;
    }
    /// Apply an angular impulse to the body
    ///
    /// does not handle with delta_time
    pub fn apply_angular_impulse(&mut self, impulse: f32) {
        self.angvel += impulse * self.inv_mass;
    }
    /// Applies a force
    ///
    /// handles with delta_time
    pub fn apply_force(&mut self, force: Vec2) {
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
    pub fn set_mass(&mut self, mass: f32) {
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

impl Default for KinematicBody2D {
    fn default() -> Self {
        Self::new()
    }
}
