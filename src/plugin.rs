//! # Main plugin
//!
//! `App.add_plugin(Physics2DPlugin)`
//!
//! Contains the plugin and stages

use crate::bodies::*;
use crate::physics_components::Transform2D;
use crate::transform_mode::TransformMode;
use crate::{broad, narrow};
use bevy::prelude::*;
use crate::normal_coll;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin;

/// General collision event that happens between 2 bodies.
pub struct CollisionEvent {
    /// First entity, will always be a non-staticbody entity
    pub entity_a: Entity,
    /// Second entity
    pub entity_b: Entity,
    /// Wether `entity_b` is a static body or not(if not then its a kinematicbody)
    pub is_b_static: bool,
    /// Normal of the collision(from `entity_a`'s perspective)
    pub normal: Vec2,
}

/// labels for the physics stages(boi i am excited stageless and also am scared of it)
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
        app: &mut App,
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
        app.add_event::<broad::ConBroadData>(); // internal event for passing data
        app.add_event::<CollisionEvent>(); // Collision event to also be viewed outside
        // Collision pairs - broad_phase_2 -> narrow_phase_2
        app.add_event::<normal_coll::CollPairKin>();
        app.add_event::<normal_coll::CollPairStatic>();
        app.add_event::<normal_coll::CollPairSensor>();

        // insert the resources
        // if `app.world().is_resource_added::<T>()` could work properly, it would be great >:( - Solved on main(so fixme on 0.6)
        app.insert_resource(TransformMode::XY);

        // Add the systems themselves for each step
        app.add_system_to_stage(
            stage::COLLISION_DETECTION,
            Transform2D::sync_from_global_transform
                .chain(sensor_clean)
                .chain(broad::broad_phase_1)
                .chain(narrow::narrow_phase_system)
                .chain(normal_coll::broad_phase_2)
                .chain(normal_coll::narrow_phase_2)
                .chain(normal_coll::ray_phase)
                .chain(Transform2D::sync_to_transform),
        );

        app.add_system(Transform2D::auto_insert_system);
    }
}

fn sensor_clean(mut query: Query<&mut Sensor>) {
    query
        .iter_mut()
        .for_each(|mut s| s.bodies.clear());
}