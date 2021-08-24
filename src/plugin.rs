//! # Main plugin
//!
//! Defining The Plugins and some other important stuff, like user facing events

use crate::bodies::*;
use crate::transform_mode::TransformMode;
use crate::{broad, narrow};
use bevy::prelude::*;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin;

/// General collision event that happens between 2 bodies.
pub struct CollisionEvent {
    /// First entity
    pub entity_a: Entity,
    /// Second entity
    pub entity_b: Entity,
    /// Wether `entity_b` is a static body or not(if not then its a kinematicbody)
    pub is_b_static: bool,
    /// Normal of the collision(from `entity_a`'s perspective)
    pub normal: Vec2,
}

/// labels for the physics stages
pub mod stage {
    pub use bevy::prelude::CoreStage;

    /// Physics step, gravity, friction, apply velocity and forces, move the bodies and such
    pub const PHYSICS_STEP: &str = "phy_physics_step";
    /// update joint constraints based on current data
    pub const JOINT_STEP: &str = "phy_joint_step";
    /// One big stage which hosts the collision detection + solve systems
    pub const COLLISION_DETECTION: &str = "phy_collision";
    /// Check for raycasts and if they detect any object in their path.
    pub const RAYCAST_DETECTION: &str = "phy_raycast_detection";
}

impl Plugin for Physics2dPlugin {
    fn build(
        &self,
        app: &mut AppBuilder,
    ) {
        // Stage order goes as follows
        // Joints step -> Physics step -> collision detection -> solve -> sync -> Raycast detection

        app.add_stage_before(
            CoreStage::Update,
            stage::PHYSICS_STEP,
            SystemStage::single_threaded(),
        )
        .add_stage_before(
            stage::PHYSICS_STEP,
            stage::JOINT_STEP,
            SystemStage::single_threaded(),
        )
        .add_stage_after(
            stage::PHYSICS_STEP,
            stage::COLLISION_DETECTION,
            SystemStage::single_threaded(),
        )
        .add_stage_after(
            stage::COLLISION_DETECTION,
            stage::RAYCAST_DETECTION,
            SystemStage::single_threaded(),
        );

        // Add the event type
        app.add_event::<broad::ConBroadData>();
        app.add_event::<CollisionEvent>();

        // insert the resources
        app.insert_resource(TransformMode::XY);

        // Add the systems themselves for each step
        app.add_system_to_stage(
            stage::COLLISION_DETECTION,
            broad::broad_phase_1
                .system()
                .chain(sensor_clean.system())
                .chain(narrow::narrow_phase_system.system()),
        );
    }
}

fn sensor_clean(mut query: Query<&mut Sensor2D>) {
    query
        .iter_mut()
        .for_each(|mut s| s.bodies.clear());
}