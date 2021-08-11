use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Linear velocity component
///
/// Default : `(0.0, 0.0)`
#[derive(Clone, Default, Reflect, Serialize, Deserialize)]
pub struct Vel(pub Vec2);

impl Vel {
    pub const ZERO : Vel = Vel(Vec2::ZERO);
}

/// Terminal velocity
///
/// Default : `(f32::INFINITY, f32::INFINITY)`
#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct TerVel(pub Vec2);

impl Default for TerVel {
    fn default() -> Self {
        Self(Vec2::splat(f32::INFINITY))
    }
}
