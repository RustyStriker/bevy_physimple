use bevy::prelude::*;
// use serde::{Deserialize, Serialize};

use crate::{
    physics_components::CollisionLayer,
    prelude::CollisionShape,
};

/// StaticBody for 2D physics(with supposedly infinite mass)
#[derive(Bundle)]
pub struct StaticBundle {
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}
