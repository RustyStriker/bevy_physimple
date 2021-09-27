//! # Hello!
//! This is my lib, check out the getting start on the repo(GETTING_STARTED.md)

mod broad;
mod narrow;
mod normal_coll;

pub mod bodies;
pub mod common;
pub mod physics_components;
pub mod plugin;
pub mod transform_mode;
pub mod shapes;

pub mod systems {
    //! Re-exports all the systems in the crate for ease of access
    pub use super::broad::broad_phase_1;
    pub use super::narrow::narrow_phase_system;
    pub use super::normal_coll::{broad_phase_2, narrow_phase_2, ray_phase, CollPairKin, CollPairSensor, CollPairStatic};
}

pub mod prelude {
    //! This module re-exports all the things you might need for 2d physics
    //! simulation.
    pub use crate::common::*;
    pub use crate::plugin::{Physics2dPlugin, CollisionEvent};
    pub use crate::physics_components::*;
    pub use crate::bodies::*;
    pub use crate::shapes::*;
}
