use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Mass
///
/// Default : `1.0`
#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct Mass {
    mass : f32,
    mass_inv : f32,
}
impl Mass {
    pub fn new(mass : f32) -> Mass {
        Mass {
            mass,
            mass_inv : mass.recip(),
        }
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn mass_inv(&self) -> f32 {
        self.mass_inv
    }
    pub fn set_mass(
        &mut self,
        mass : f32,
    ) {
        self.mass = mass;
        self.mass_inv = mass.recip();
    }
}
impl Default for Mass {
    fn default() -> Self {
        Self {
            mass : 1.0,
            mass_inv : 1.0,
        }
    }
}

/// Friction multiplier
///
/// if no `FrictionMult` is provided, a default value of `1.0` will be used
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct FrictionMult(pub f32);
impl Default for FrictionMult {
    fn default() -> Self {
        Self(1.0)
    }
}
impl Deref for FrictionMult {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FrictionMult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
