use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Linear velocity tuple component(access the value using `Vel.0`)
///
/// Required for a continuous collision kinematic body 
///
/// Default: `(0.0, 0.0)`
#[derive(Clone, Default, Reflect, Serialize, Deserialize, Component)]
pub struct Vel(pub Vec2);

impl Vel {
    pub const ZERO: Vel = Vel(Vec2::ZERO);
}