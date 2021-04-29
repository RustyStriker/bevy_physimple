//! This module provides the primitives and systems for 2d physics simulation.
//!
//! For examples, see the root of the crate.

use bevy::math::*;
use bevy::prelude::*;

use crate::common::*;
use crate::bodies::*;
use crate::shapes::*;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin {
    /// Global settings for the physics calculations
    settings : Option<PhysicsSettings>
}
impl Default for Physics2dPlugin {
    fn default() -> Self {
        Physics2dPlugin {
            settings : None,
        }
    }
}

#[derive(Clone, Debug,)]
pub struct PhysicsSettings {
    /// How strong the friction is
    ///
    /// Currently a number between (0.0 - 1.0) where 1.0 is no friction
    pub friction : f32,
    /// The direction in which friction wont exist
    pub friction_normal : Vec2,
    /// Gravity direction and strength(up direction is opposite to gravity)
    pub gravity : Vec2,
    pub translation_mode : TranslationMode,
    pub rotatoin_mode : RotationMode,
}
impl Default for PhysicsSettings {
    fn default() -> Self {
        PhysicsSettings {
            friction : 0.9,
            friction_normal : Vec2::Y,
            gravity : Vec2::new(0.0,-540.0),
            translation_mode : TranslationMode::AxesXY,
            rotatoin_mode : RotationMode::AxisZ,
        }
    }
}


pub mod stage {
    #[doc(hidden)]
    pub use bevy::prelude::CoreStage;

    pub const COLLIDING_JOINT: &str = "colliding_joint";
    pub const PHYSICS_STEP: &str = "physics_step";
    pub const BROAD_PHASE: &str = "broad_phase";
    pub const NARROW_PHASE: &str = "narrow_phase";
    pub const PHYSICS_SOLVE: &str = "physics_solve";
    pub const RIGID_JOINT: &str = "rigid_joint";
    pub const SYNC_TRANSFORM: &str = "sync_transform";
}

impl Plugin for Physics2dPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let settings = self.settings.clone().unwrap_or(PhysicsSettings::default());

        app
            .insert_resource(settings)
            .add_stage_before(CoreStage::Update, stage::PHYSICS_STEP, SystemStage::single_threaded())
            .add_stage_before(stage::PHYSICS_STEP, stage::COLLIDING_JOINT,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_STEP, stage::BROAD_PHASE,SystemStage::single_threaded())
            .add_stage_after(stage::BROAD_PHASE, stage::NARROW_PHASE,SystemStage::single_threaded())
            .add_stage_after(stage::NARROW_PHASE, stage::PHYSICS_SOLVE,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_SOLVE, stage::RIGID_JOINT,SystemStage::single_threaded())
            .add_stage_after(stage::RIGID_JOINT, stage::SYNC_TRANSFORM,SystemStage::single_threaded());

        // Add the event type
        app.add_event::<AABBCollisionEvent>();

        // Add the systems themselves for each step
        app.add_system_to_stage(stage::PHYSICS_STEP, physics_step_system.system())
            .add_system_to_stage(stage::NARROW_PHASE, aabb_collision_detection_system.system())
            .add_system_to_stage(stage::PHYSICS_SOLVE, aabb_solve_system.system())
            .add_system_to_stage(stage::SYNC_TRANSFORM, sync_transform_system.system());
        // TODO Recreate the Joint systems

    }
}

fn get_child_shapes<'a>(shapes : &'a Query<&AABB>, children : &Children) -> Option<AABB> {
    for &e in children.iter() {
        if let Ok(shape) = shapes.get_component::<AABB>(e) {
            return Some(*shape);
        }
    }
    None
}

#[cfg(test)]
mod aabb_collision_tests {
    use super::*;
    #[test]
    fn xpen_left() {
        let aabb = AABB::new(Vec2::new(10.0,10.0));

        let res = get_aabb_collision(
            aabb,
            aabb,
            Vec2::ZERO, 
            Vec2::new(18.0,5.0)
        );
        assert_eq!(Some(Vec2::new(-2.0,0.0)), res);
    }
    #[test]
    fn ypen_up() {
        let aabb = AABB::new(Vec2::new(10.0,10.0));

        let res = get_aabb_collision(
            aabb, 
            aabb, 
            Vec2::ZERO, 
            Vec2::new(5.0,-18.0)
        );

        assert_eq!(Some(Vec2::new(0.0,2.0)),res);
    }
}
/// Checks for collision between 2 AABB objects and returns the penetration(of a in b) if existing
fn get_aabb_collision(a : AABB, b : AABB, a_pos : Vec2, b_pos : Vec2) -> Option<Vec2> {
    let amin = a_pos - a.extents;
    let amax = a_pos + a.extents;
    let bmin = b_pos - b.extents;
    let bmax = b_pos + b.extents;

    // Check for a general collision
    let coll_x = amax.x >= bmin.x && bmax.x >= amin.x;
    let coll_y = amax.y >= bmin.y && bmax.y >= amin.y;

    if coll_x && coll_y {
        // Search for the least penetrated axis
        let xpen_left = (amax.x - bmin.x).abs();
        let xpen_right = (amin.x - bmax.x).abs();
        let ypen_up = (amin.y - bmax.y).abs();
        let ypen_down = (amax.y - bmin.y).abs();

        let min = xpen_left.min(xpen_right).min(ypen_up).min(ypen_down);

        if min == xpen_left {
            Some(Vec2::new(-xpen_left,0.0))
        }
        else if min == xpen_right {
            Some(Vec2::new(xpen_right,0.0))
        }
        else if min == ypen_up {
            Some(Vec2::new(0.0,ypen_up))
        }
        else if min == ypen_down {
            Some(Vec2::new(0.0,-ypen_down))
        }
        else {
            panic!("Something went really wrong, max isnt equal any of them")
        }
    }
    else {
        None
    }
}

/// Temp struct for aabb collision event
#[derive(Clone, Debug)]
pub struct AABBCollisionEvent {
    pub entity_a : Entity,
    pub entity_b : Entity,
    /// Penetration of a in b, can get normal out of it
    pub penetration : Vec2,
    /// If the collision happened with a static body
    pub with_static : bool
}

fn aabb_collision_detection_system (
    // mut commands : Commands,
    q_kinematic : Query<(Entity, &KinematicBody2D, &Children)>,
    q_static : Query<(Entity, &StaticBody2D, &Children)>,
    mut q_sensors : Query<(Entity, &mut Sensor2D, &Children)>,
    shapes : Query<&AABB>,
    mut writer : EventWriter<AABBCollisionEvent>,
) {
    // Clear all the sensors overlapping parts
    q_sensors.iter_mut().for_each(|(_,mut s,_)| s.overlapping_bodies.clear());
    
    let mut passed : Vec<(Entity, &KinematicBody2D, AABB)> = Vec::new();

    // Go through all the kinematic bodies
    for (entity, body, children) in q_kinematic.iter() {
        // Gather all the shape children(colliders...)
        let collider = match get_child_shapes(&shapes, &children) {
            Some(c) => c,
            None => continue,
        };
        
        // Go through the static bodies and check for collisions
        for (se, sb, children) in q_static.iter() {
            let sc = match get_child_shapes(&shapes, &children) {
                Some(c) => c,
                None => continue,
            };
            // Check for collision here
            if let Some(pen) = get_aabb_collision( collider,  sc,  body.position,  sb.position) {
                writer.send(
                    AABBCollisionEvent {
                        entity_a : entity,
                        entity_b : se,
                        penetration : pen,
                        with_static : true
                    }
                );
            }
        }
        // Go through sensors to know who is inside the sensor
        // we iter_mut because we want to do sensor.overlapping_bodies.push(entity) for each entity
        // that is overlapping with the sensor
        for (_, mut sensor, children) in q_sensors.iter_mut() {
            let sc = match get_child_shapes(&shapes, &children) {
                Some(c) => c,
                None => continue,
            };
            // Check for collision here
            if let Some(_) = get_aabb_collision(collider, sc, body.position, sensor.position) {
                sensor.overlapping_bodies.push(entity);
            }
        }
        // Go through all the kinematic bodies we passed already
        for (ke, ob, oc) in passed.iter() {
            // check for collisions here...
            if let Some(pen) = get_aabb_collision( collider,  *oc,  body.position,  ob.position) {
                writer.send(
                    AABBCollisionEvent {
                        entity_a : entity,
                        entity_b : *ke,
                        penetration : pen,
                        with_static : false
                    }
                );
            }
        }
        
        passed.push((entity, body, collider));
    }
}

fn aabb_solve_system (
    mut collisions : EventReader<AABBCollisionEvent>,
    mut bodies : Query<&mut KinematicBody2D>,
    staticbodies : Query<&StaticBody2D>,
) {
    for coll in collisions.iter() {
        
        let normal = coll.penetration.normalize();
        
        if coll.with_static {
            let mut a = match bodies.get_component_mut::<KinematicBody2D>(coll.entity_a) {
                Ok(b) => b,
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_a);
                    continue;
                }
            };
            let with_sb = match staticbodies.get_component::<StaticBody2D>(coll.entity_b) {
                Ok(b) => b,
                Err(_) => {
                    eprintln!("Couldnt get StaticBody2D of entity {:?}", coll.entity_b);
                    continue;
                }
            };

            // Check for floor/wall/ceil collision(maybe change it later to only static bodies?)
            // TODO Switch to user defined values
            check_on_stuff(&mut a, normal);

            // if colliding with a static object, just undo the penetration and slide across the normal(aka pen direction)
            // TODO Maybe add a step functionality here?

            if a.linvel.signum() != coll.penetration.signum() {
                let project = a.linvel.project(normal);
                let slide = a.linvel - project; // This is pretty much how slide works

                let linvel = slide - project * with_sb.bounciness;

                a.linvel = linvel;
                a.position += coll.penetration;
            }
        }
        else {
            // Collision with another body
            let b = match bodies.get_component::<KinematicBody2D>(coll.entity_b) {
                Ok(b) => b,
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_b);
                    continue;
                }
            };
            let a = match bodies.get_component::<KinematicBody2D>(coll.entity_a) {
                Ok(a) => a,
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_a);
                    continue;
                }
            };

            let sum_recip = (a.mass + b.mass).recip();
            let br = b.linvel * b.mass;
            let ar = a.linvel * a.mass;
            let rv = br * sum_recip - ar * sum_recip;

            let impulse = rv.project(normal);

            // explicit drop to convey they are not usable anymore because we borrow_mut bodies just below 
            drop(a);
            drop(b);
            match bodies.get_component_mut::<KinematicBody2D>(coll.entity_b) {
                Ok(mut b) => {
                    b.dynamic_acc -= impulse;
                    check_on_stuff(&mut b, -normal);

                    if b.linvel.signum() != -coll.penetration.signum() {
                        b.position -= coll.penetration;
                        b.linvel = b.linvel.slide(normal);
                    }
                },
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_b);
                    continue;
                }
            };
            match bodies.get_component_mut::<KinematicBody2D>(coll.entity_a) {
                Ok(mut a) => {
                    a.dynamic_acc += impulse;
                    check_on_stuff(&mut a, -normal);
                    if a.linvel.signum() != coll.penetration.signum() {
                        a.position += coll.penetration;
                        a.linvel = a.linvel.slide(normal);
                    }
                    check_on_stuff(&mut a, normal);
                },
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_a);
                    continue;
                }
            };
        }
    }
}

fn check_on_stuff(body : &mut KinematicBody2D, normal : Vec2) {
    // Check for floor/wall/ceil collision(maybe change it later to only static bodies?)
    // TODO Switch to user defined values
    const FLOOR_ANGLE : f32 = 0.7;
    let up = Vec2::new(0.0,1.0);
    let dot = up.dot(normal);

    if dot >= FLOOR_ANGLE {
        body.on_floor = Some(normal);
    }
    if dot.abs() < FLOOR_ANGLE {
        body.on_wall = Some(normal);
    }
    if dot <= -FLOOR_ANGLE {
        body.on_ceil = Some(normal);
    }
}

/// apply gravity, movement, rotation, forces, friction and other stuff as well
fn physics_step_system (
    time : Res<Time>,
    physics_sets : Res<PhysicsSettings>,
    mut query : Query<&mut KinematicBody2D>,
) {
    let delta = time.delta_seconds();
    let gravity = physics_sets.gravity;

    for mut body in query.iter_mut() {
        if !body.active {
            continue;
        }

        let accelerating = body.accumulator.length_squared() > 0.1 && body.dynamic_acc.length_squared() > 0.1;

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
        { // Brackets because we no longer need those variables
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
        let position = body.position + body.linvel * delta;
        body.position = position;

        let rotation = body.rotation + body.angvel * delta;
        body.rotation = rotation;

        // Apply friction
        let friction_normal = physics_sets.friction_normal;
        let vel_proj = body.linvel.project(friction_normal);
        let vel_slided = body.linvel - vel_proj; // This is pretty much how project works
        body.linvel = vel_proj + vel_slided * physics_sets.friction;


        // TODO better friciton based on gravity orientation please
        body.angvel *= physics_sets.friction;

        // Reset on_* variables
        body.on_floor = None;
        body.on_wall = None;
        body.on_ceil = None;
    }
}

/// The plane on which to translate the 2d position into 3d coordinates.
#[derive(Debug, Clone, Copy)]
pub enum TranslationMode {
    AxesXY,
    AxesXZ,
    AxesYZ,
}

impl Default for TranslationMode {
    fn default() -> Self {
        Self::AxesXY
    }
}

/// The axis on which to rotate the 2d rotation into a 3d quaternion.
#[derive(Debug, Clone, Copy)]
pub enum RotationMode {
    AxisX,
    AxisY,
    AxisZ,
}

impl Default for RotationMode {
    fn default() -> Self {
        Self::AxisZ
    }
}

pub fn sync_transform_system (
    translation_mode: Res<TranslationMode>,
    rotation_mode: Res<RotationMode>,
    mut query : QuerySet<(
        Query<(&Sensor2D, &mut Transform)>,
        Query<(&KinematicBody2D, &mut Transform)>,
        Query<(&StaticBody2D, &mut Transform)>
    )>
) {
    let sync = move | pos : Vec2, rot : f32, transform : &mut Transform | {
        match *translation_mode {
            TranslationMode::AxesXY => {
                transform.translation.x = pos.x;
                transform.translation.y = pos.y;
            }
            TranslationMode::AxesXZ => {
                transform.translation.x = pos.x;
                transform.translation.z = pos.y;
            }
            TranslationMode::AxesYZ => {
                transform.translation.y = pos.x;
                transform.translation.z = pos.y;
            }
        }
        match *rotation_mode {
            RotationMode::AxisX => {
                transform.rotation = Quat::from_rotation_x(rot);
            }
            RotationMode::AxisY => {
                transform.rotation = Quat::from_rotation_y(rot);
            }
            RotationMode::AxisZ => {
                transform.rotation = Quat::from_rotation_z(rot);
            }
        }
    };
    for (body, mut t) in query.q0_mut().iter_mut() {
        sync(body.position,body.rotation, &mut t);
    }
    for (body, mut t) in query.q1_mut().iter_mut() {
        sync(body.position,body.rotation, &mut t);
    }
    for (body, mut t) in query.q2_mut().iter_mut() {
        sync(body.position,body.rotation, &mut t);
    }
}