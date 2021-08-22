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