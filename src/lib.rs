pub mod common;
pub mod plugin;
pub mod bodies;
pub mod shapes;
mod narrow;
mod broad;

pub mod prelude {
    //! This module re-exports all the things you might need for 2d physics
    //! simulation.
    pub use crate::common::*;
    pub use crate::plugin::{
        PhysicsSettings,
        Physics2dPlugin, 
        TransformMode,
    };
    // TODO Maybe restrict it a bit more?
    pub use crate::bodies::*;
    pub use crate::shapes::*;
}

