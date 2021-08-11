use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Angular velocity component
#[derive(Clone, Default, Reflect, Serialize, Deserialize)]
pub struct AngVel(pub f32);

impl AngVel {
    pub const ZERO : AngVel = AngVel(0.0);
}
impl Deref for AngVel {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AngVel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Terminal angular velocity
///
/// Default : `f32::INFINITY`
#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct TerAngVel(pub f32);
impl Default for TerAngVel {
    fn default() -> Self {
        Self(f32::INFINITY)
    }
}
