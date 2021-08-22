use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// TODO make a SensorBundle...

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Sensor2D {
    /// Holds the entities which overlap with the sensor.
    pub bodies : Vec<Entity>,
}

impl Sensor2D {
    pub fn new() -> Self {
        Sensor2D {
            bodies : Vec::with_capacity(5),
        }
    }
}
impl Default for Sensor2D {
    fn default() -> Self {
        Self::new()
    }
}
