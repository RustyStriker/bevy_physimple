use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{physics_components::CollisionLayer, prelude::CollisionShape};

#[derive(Bundle, Default)]
pub struct SensorBundle {
    pub sensor : Sensor,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}

/**
    # Sensor

    A Sensor will check each frame what kinematic entites overlap it,
    and store their `Entity` in the `Sensor.bodies` Vec.

    NOTE: "kinematic entities" qualifies as `Without<StaticBody>, Without<Sensor>`
*/
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Sensor {
    /// Holds the entities which overlap with the sensor.
    pub bodies : Vec<Entity>,
}

impl Sensor {
    pub fn new() -> Self {
        Sensor {
            bodies : Vec::with_capacity(5),
        }
    }
}
impl Default for Sensor {
    fn default() -> Self {
        Self::new()
    }
}
