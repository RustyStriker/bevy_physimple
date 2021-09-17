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

pub mod prelude {
    //! This module re-exports all the things you might need for 2d physics
    //! simulation.
    pub use crate::common::*;
    pub use crate::plugin::{Physics2dPlugin, CollisionEvent};
    pub use crate::physics_components::*;
    pub use crate::bodies::*;
    pub use crate::shapes::*;
}
