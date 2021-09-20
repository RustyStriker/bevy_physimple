use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    physics_components::CollisionLayer,
    prelude::CollisionShape,
};

/// This is a marker component
///
/// Static bodies have a couple of interesting differences from non-Static bodies:
/// - Static bodies are being calculated against with the continuous collision algorithms
/// - Static bodies aren't being checked for sensor collision
/// - Static bodies do not move in case of collision
///
/// So generally, mark as much Staticbodies as possible!
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct StaticBody;

/// StaticBody for 2D physics(with supposedly infinite mass)
#[derive(Bundle, Default)]
pub struct StaticBundle {
    pub marker : StaticBody,
    pub shape : CollisionShape,
    pub coll_layer : CollisionLayer,
}
