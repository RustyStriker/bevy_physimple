//! # Main plugin
//!
//! Defining The Plugins and some other important stuff, like user facing events

use crate::bodies::*;
use crate::shapes::*;
use crate::{
    broad::{self, ObbData, ObbDataKinematic},
    common::*,
    narrow,
};
use bevy::prelude::*;
use std::f32::consts::PI;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin {
    /// Global settings for the physics calculations
    settings : PhysicsSettings,
}
impl Default for Physics2dPlugin {
    fn default() -> Self {
        Physics2dPlugin {
            settings : PhysicsSettings::default(),
        }
    }
}

/// Settings for the physics systems to use
///
/// usually the defaults should be enough, besides a couple of parameters(friction, gravity, ang_friction)
#[derive(Clone, Debug)]
pub struct PhysicsSettings {
    /// How strong the force of friction is(default - 400.0)
    pub friction : f32,
    /// The direction in which friction wont exist
    ///
    /// or the normal vector for the plane in which friction does exists(should be `gravity.normalize()`)
    pub friction_normal : Vec2,
    /// Friction on the angular velocity in radians
    pub ang_friction : f32,

    /// Gravity direction and strength(up direction is opposite to gravity)
    pub gravity : Vec2,

    pub transform_mode : TransformMode,
    /// What angles are considered floor/wall/ceilling
    ///
    /// a number between 0-1 representing 'normal.dot(-gravity)'
    ///
    /// floor >= floor_angle // wall.abs() < floor_angle // ceil <= -floor_angle
    ///
    /// Defaults to 0.7
    pub floor_angle : f32,
}
impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings {
            friction : 400.0,
            friction_normal : Vec2::Y,
            ang_friction : PI,
            gravity : Vec2::new(0.0, -540.0),
            transform_mode : TransformMode::XY,
            floor_angle : 0.7,
        }
    }
}

/// Which plane acts as the XY plane, rotation axis is the perpendicular axis
#[derive(Debug, Clone, Copy)]
pub enum TransformMode {
    XY,
    XZ,
    YZ,
}
impl TransformMode {
    /// Returns the position from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_position(
        &self,
        transform : &GlobalTransform,
    ) -> Vec2 {
        let t = transform.translation;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the rotation from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_rotation(
        &self,
        transform : &GlobalTransform,
    ) -> f32 {
        let t = transform.rotation;

        match self {
            TransformMode::XY => t.z,
            TransformMode::XZ => t.y,
            TransformMode::YZ => t.x,
        }
    }
    /// Returns the scale from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_scale(
        &self,
        transform : &GlobalTransform,
    ) -> Vec2 {
        let t = transform.scale;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the position from a given `&Transform` and `TransformMode`
    pub fn get_position(
        &self,
        transform : &Transform,
    ) -> Vec2 {
        let t = transform.translation;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the rotation from a given `&Transform` and `TransformMode`
    pub fn get_rotation(
        &self,
        transform : &Transform,
    ) -> f32 {
        let t = transform.rotation;

        match self {
            TransformMode::XY => t.z,
            TransformMode::XZ => t.y,
            TransformMode::YZ => t.x,
        }
    }
    /// Returns the scale from a given `&Transform` and `TransformMode`
    pub fn get_scale(
        &self,
        transform : &Transform,
    ) -> Vec2 {
        let t = transform.scale;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Sets position based on `TransformMode`
    pub fn set_position(
        &self,
        transform : &mut Transform,
        pos : Vec2,
    ) {
        let t = transform.translation;

        transform.translation = match self {
            TransformMode::XY => Vec3::new(pos.x, pos.y, t.z),
            TransformMode::XZ => Vec3::new(pos.x, t.y, pos.y),
            TransformMode::YZ => Vec3::new(t.x, pos.x, pos.y),
        };
    }
    /// Sets rotation based on `TransformMode` (erase previus rotation)
    pub fn set_rotation(
        &self,
        transform : &mut Transform,
        rot : f32,
    ) {
        // TODO make it persist the other axis rotations, i dont understand quaternions
        transform.rotation = match self {
            TransformMode::XY => Quat::from_rotation_z(rot),
            TransformMode::XZ => Quat::from_rotation_y(rot),
            TransformMode::YZ => Quat::from_rotation_x(rot),
        }
    }
}
/// General collision event that happens between 2 bodies.
pub struct CollisionEvent {
    /// First entity
    pub entity_a : Entity,
    /// Second entity
    pub entity_b : Entity,
    /// Wether `entity_b` is a static body or not(if not then its a kinematicbody)
    pub is_b_static : bool,
    /// Normal of the collision(from `entity_a`'s perspective)
    pub normal : Vec2,
}

/// labels for the physics stages
pub mod stage {
    pub use bevy::prelude::CoreStage;

    /// update joint constraints based on current data
    pub const JOINT_STEP : &str = "phy_joint_step";
    /// Resets sensor collision data for the next step
    pub const SENSOR_RESET_STEP : &str = "phy_sensor_reset_step";
    /// Physics step, gravity, friction, apply velocity and forces, move the bodies and such
    pub const PHYSICS_STEP : &str = "phy_physics_step";

    pub const CAPTURE_STEP : &str = "phy_capture_step";
    /// Check for collisions between objects, emitting events with AABBCollisionEvent(should be replaced later tho)
    pub const COLLISION_DETECTION : &str = "phy_collision_detection";
    /// Solve each collision and apply forces based on collision
    pub const PHYSICS_SOLVE : &str = "phy_solve";
    /// Check for raycasts and if they detect any object in their path.
    pub const RAYCAST_DETECTION : &str = "phy_raycast_detection";
}

impl Plugin for Physics2dPlugin {
    fn build(
        &self,
        app : &mut AppBuilder,
    ) {
        let settings = self.settings.clone();

        // Stage order goes as follows
        // Joints step -> Physics step -> collision detection -> solve -> sync -> Raycast detection

        app.insert_resource(settings)
            .add_stage_before(
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
                stage::SENSOR_RESET_STEP,
                SystemStage::single_threaded(),
            )
            .add_stage_after(
                stage::SENSOR_RESET_STEP,
                stage::CAPTURE_STEP,
                SystemStage::parallel(),
            )
            .add_stage_after(
                stage::CAPTURE_STEP,
                stage::COLLISION_DETECTION,
                SystemStage::single_threaded(),
            )
            .add_stage_after(
                stage::COLLISION_DETECTION,
                stage::PHYSICS_SOLVE,
                SystemStage::single_threaded(),
            )
            .add_stage_after(
                stage::PHYSICS_SOLVE,
                stage::RAYCAST_DETECTION,
                SystemStage::single_threaded(),
            );

        // Add the event type
        app.add_event::<ObbData>();
        app.add_event::<ObbDataKinematic>();
        app.add_event::<CollisionEvent>();

        // Add the systems themselves for each step
        app.add_system_to_stage(stage::PHYSICS_STEP, physics_step_system.system())
            .add_system_to_stage(
                stage::CAPTURE_STEP,
                broad::broad_phase_system::<Square>.system(),
            )
            .add_system_to_stage(
                stage::CAPTURE_STEP,
                broad::broad_phase_system::<Circle>.system(),
            )
            .add_system_to_stage(
                stage::COLLISION_DETECTION,
                narrow::narrow_phase_system.system(),
            );
        // TODO Recreate the Joint systems
    }
}

/// apply gravity, movement, rotation, forces, friction and other stuff as well
fn physics_step_system(
    time : Res<Time>,
    physics_sets : Res<PhysicsSettings>,
    mut query : Query<(&mut KinematicBody2D, &mut Transform)>,
) {
    let delta = time.delta_seconds();
    let gravity = physics_sets.gravity;
    let trans_mode = physics_sets.transform_mode;

    for (mut body, mut transform) in query.iter_mut() {
        if !body.active {
            continue;
        }

        let accelerating =
            body.accumulator.length_squared() > 0.1 || body.dynamic_acc.length_squared() > 0.1;

        // Gravity
        if body.mass > f32::EPSILON {
            body.linvel += gravity * delta;
        }
        // Apply forces and such
        let linvel = body.linvel + body.accumulator * delta;
        let linvel = linvel + body.dynamic_acc;
        body.linvel = linvel;
        body.accumulator = Vec2::ZERO;
        body.dynamic_acc = Vec2::ZERO;

        // Terminal velocity cheks(per axis)
        {
            // Brackets because we no longer need those variables
            let vel = body.linvel;
            let limit = body.terminal;
            if vel.x.abs() > limit.x {
                body.linvel.x = vel.x.signum() * limit.x;
            }
            if vel.y.abs() > limit.y {
                body.linvel.y = vel.y.signum() * limit.y;
            }
            let vel = body.angvel;
            let limit = body.ang_terminal;
            if vel.abs() > limit {
                body.angvel = vel.signum() * limit;
            }
        }
        // Apply movement and rotation
        body.inst_linvel = body.linvel * delta;
        let position = trans_mode.get_position(&transform) + body.inst_linvel;
        trans_mode.set_position(&mut transform, position);

        let rotation = trans_mode.get_rotation(&transform) + body.angvel * delta;
        trans_mode.set_rotation(&mut transform, rotation);

        // Apply friction
        if !accelerating {
            let friction_normal = physics_sets.friction_normal;
            let vel_proj = body.linvel.project(friction_normal);
            let mut vel_slided = body.linvel - vel_proj; // This is pretty much how project works

            let vel_slided_len = vel_slided.length(); // We keep it to normalize the vector later
            let friction_strength = physics_sets.friction * body.friction_mult * delta; // Current frame's friction
            if vel_slided_len <= friction_strength {
                vel_slided = Vec2::ZERO;
            }
            else {
                vel_slided -= (vel_slided / vel_slided_len) * friction_strength;
                //             /\~~~~~~~~~~~~~~~~~~~~~~~~/\ normalized vel_slided
            }

            body.linvel = vel_proj + vel_slided; // Apply the new friction values to linvel
        }
        let angular_friction = physics_sets.ang_friction * delta;
        if body.angvel.abs() < angular_friction {
            body.angvel = 0.0;
        }
        else {
            let sign = body.angvel.signum();
            body.angvel -= sign * angular_friction;
        }

        // Reset on_* variables
        body.on_floor = None;
        body.on_wall = None;
        body.on_ceil = None;
    }
}
