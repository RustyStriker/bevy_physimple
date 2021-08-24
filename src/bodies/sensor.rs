use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{physics_components::CollisionLayer, prelude::CollisionShape};

// TODO make a SensorBundle...
#[derive(Bundle)]
pub struct SensorBundle {
    pub sensor : Sensor,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}


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
