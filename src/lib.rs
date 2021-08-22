pub mod bodies;
mod broad;
pub mod common;
mod narrow;
pub mod physics_components;
pub mod plugin;
pub mod transform_mode;
pub mod shapes;

pub mod prelude {
    //! This module re-exports all the things you might need for 2d physics
    //! simulation.
    pub use crate::common::*;
    pub use crate::plugin::Physics2dPlugin;
    pub use crate::bodies::*;
    pub use crate::shapes::*;
}
