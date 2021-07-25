//! # Main plugin
//!
//! Defining The Plugins and some other important stuff, like user facing events

use crate::bodies::*;
use crate::physics_components::angular_velocity::AngVel;
use crate::physics_components::angular_velocity::TerAngVel;
use crate::physics_components::velocity::TerVel;
use crate::physics_components::{physical_properties::{FrictionMult, Mass}, velocity::Vel};
use crate::settings::{AngFriction, Friction, Gravity, TransformMode};
use crate::shapes::*;
use crate::{
    broad,
    common::*,
    narrow,
};
use bevy::prelude::*;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin;


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
        // Stage order goes as follows
        // Joints step -> Physics step -> collision detection -> solve -> sync -> Raycast detection

        app
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
        app.add_event::<broad::BroadData>();
        app.add_event::<CollisionEvent>();

        // insert the resources
        crate::settings::insert_physics_resources(app);

        // Add the systems themselves for each step
        app
            .add_system_to_stage(
                stage::CAPTURE_STEP,
                broad::broad_phase_1.system(),
            )
            .add_system_to_stage(
                stage::COLLISION_DETECTION,
                narrow::narrow_phase_system.system(),
            );

        app.add_system_set_to_stage(stage::PHYSICS_STEP,
            SystemSet::new()
                .with_system(global_gravity_system.system())
                .with_system(friction_system.system())
                .with_system(ang_friction_system.system())
                .with_system(terminal_vel_system.system())
                .with_system(terminal_ang_vel_system.system())
                .with_system(kinematic_pre_update_system.system())
                .with_system(apply_ang_vel_system.system())
        );
        // TODO Recreate the Joint systems
    }
}

fn global_gravity_system(
    time : Res<Time>,
    gravity : Res<Gravity>,
    mut query : Query<(&mut Vel, &Mass)>,
) {
    let delta = time.delta_seconds();

    for (mut vel, mass) in query.iter_mut() {
        if mass.mass() > f32::EPSILON { // Not 0
            vel.0 += gravity.0 * delta;
        }
    }
}

fn friction_system(
    time : Res<Time>,
    friction : Res<Friction>,
    mut query : Query<(&mut Vel, &FrictionMult)>,
) {
    let delta = time.delta_seconds();

    for (mut vel, mult) in query.iter_mut() {
        // Holy shit this is ugly... it doesnt look TOO bad but  boi its ugly af
        let vel_proj = vel.0.project(friction.normal);
        let mut vel_slided = vel.0 - vel_proj; // This is pretty much how project works

        let vel_slided_len = vel_slided.length(); // We keep it to normalize the vector later
        let friction_strength = friction.strength * mult.0 * delta; // Current frame's friction
        if vel_slided_len <= friction_strength {
            vel_slided = Vec2::ZERO;
        }
        else {
            vel_slided -= (vel_slided / vel_slided_len) * friction_strength;
            //             /\~~~~~~~~~~~~~~~~~~~~~~~~/\ normalized vel_slided
        }

        vel.0 = vel_proj + vel_slided; // Apply the new friction values to vel
    }
}

fn ang_friction_system(
    time : Res<Time>,
    ang_fric : Res<AngFriction>,
    mut query: Query<&mut AngVel>,
) {
    let strength = time.delta_seconds() * ang_fric.0;

    for mut v in query.iter_mut() {
        if v.0 < strength {
            v.0 = 0.0;
        }
        else {
            let sign = v.0.signum();
            v.0 -= sign * strength;
        }
    }
}

fn terminal_vel_system(
    mut query : Query<(&mut Vel, &TerVel)>,
) {
    for (mut vel, ter) in query.iter_mut() {
        let v = vel.0;
        let limit = ter.0;
        if v.x.abs() > limit.x {
            vel.0.x = v.x.signum() * limit.x;
        }
        if v.y.abs() > limit.y {
            vel.0.y = v.y.signum() * limit.y;
        }
    }
}

fn terminal_ang_vel_system(
    mut query : Query<(&mut AngVel, &TerAngVel)>,
) {
    for (mut vel, ter) in query.iter_mut() {
        if vel.0.abs() > ter.0 {
            let sign = vel.0.signum();
            vel.0 = sign * ter.0;
        }
    }
}

fn kinematic_pre_update_system(
    mut query : Query<&mut KinematicBody2D>,
) {
    for mut k in query.iter_mut() {
        // Reset collision data
        k.on_floor = None;
        k.on_wall = None;
        k.on_ceil = None;

    }
}

fn apply_ang_vel_system(
    time : Res<Time>,
    trans_mode : Res<TransformMode>,
    mut query : Query<(&AngVel, &mut Transform)>,
) {
    let delta = time.delta_seconds();

    for (av, mut t) in query.iter_mut() {
        let angle = trans_mode.get_rotation(&t);
        trans_mode.set_rotation(&mut t, angle + av.0 * delta);
    }
}



/*
    Simply adding to movement
    
    Applying changes based on movement

    Limits/cleanup of movement

    Movement/update kinematic data

    Collision detection
*/
