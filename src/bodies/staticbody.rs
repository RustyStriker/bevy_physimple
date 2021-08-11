use bevy::prelude::*;
// use serde::{Deserialize, Serialize};

use crate::{
    physics_components::CollisionLayer,
    prelude::{CollisionShape, Obv},
};

// TODO maybe make a `StaticBody` marker component(or with  bounciness or something)?

/// StaticBody for 2D physics(with supposedly infinite mass)
#[derive(Bundle)]
pub struct StaticBundle {
    pub shape : CollisionShape,
    pub obv : Obv,
    pub coll_layer : CollisionLayer,
}
