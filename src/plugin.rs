use std::f32::consts::PI;

use bevy::{ecs::component::Component, prelude::*};

use crate::common::*;
use crate::bodies::*;
use crate::shapes::*;

/// Physics plugin for 2D physics
pub struct Physics2dPlugin {
    /// Global settings for the physics calculations
    settings : PhysicsSettings
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
#[derive(Clone, Debug,)]
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
            gravity : Vec2::new(0.0,-540.0),
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
    pub fn get_position(&self, transform : &GlobalTransform) -> Vec2 {
        let t = transform.translation;
        
        match self {
            TransformMode::XY => Vec2::new(t.x,t.y),
            TransformMode::XZ => Vec2::new(t.x,t.z),
            TransformMode::YZ => Vec2::new(t.y,t.z),
        }
    }
    /// Returns the rotation from a given `&GlobalTransform` and `TransformMode`
    pub fn get_rotation(&self, transform : &GlobalTransform) -> f32 {
        let t = transform.rotation;
        
        match self {
            TransformMode::XY => t.z,
            TransformMode::XZ => t.y,
            TransformMode::YZ => t.x,
        }
    }
    /// Returns the scale from a given `&GlobalTransform` and `TransformMode`
    pub fn get_scale(&self, transform : &GlobalTransform) -> Vec2 {
        let t = transform.scale;

        match self {
            TransformMode::XY => Vec2::new(t.x,t.y),
            TransformMode::XZ => Vec2::new(t.x,t.z),
            TransformMode::YZ => Vec2::new(t.y,t.z),
        }
    }
    /// Sets position based on `TransformMode`
    pub fn set_position(&self, transform : &mut GlobalTransform, pos : Vec2) {
        let t = transform.translation;

        transform.translation = match self {
            TransformMode::XY => Vec3::new(pos.x,pos.y,t.z),
            TransformMode::XZ => Vec3::new(pos.x,t.y, pos.y),
            TransformMode::YZ => Vec3::new(t.x, pos.x, pos.y),
        };
    }

    /// Sets rotation based on `TransformMode` (erase previus rotation)
    pub fn set_rotation(&self, transform : &mut GlobalTransform, rot : f32) {
        // TODO make it persist the other axis rotations, i dont understand quaternions
        transform.rotation = match self {
            TransformMode::XY => Quat::from_rotation_z(rot),
            TransformMode::XZ => Quat::from_rotation_y(rot),
            TransformMode::YZ => Quat::from_rotation_x(rot),
        }
    }
}

/// labels for the physics stages
pub mod stage {
    pub use bevy::prelude::CoreStage;

    /// update joint constraints based on current data
    pub const JOINT_STEP: &str = "phy_joint_step";
    /// Resets sensor collision data for the next step
    pub const SENSOR_RESET_STEP : &str = "phy_sensor_reset_step";
    /// Physics step, gravity, friction, apply velocity and forces, move the bodies and such
    pub const PHYSICS_STEP: &str = "phy_physics_step";
    /// Check for collisions between objects, emitting events with AABBCollisionEvent(should be replaced later tho)
    pub const COLLISION_DETECTION: &str = "phy_collision_detection";
    /// Solve each collision and apply forces based on collision
    pub const PHYSICS_SOLVE: &str = "phy_solve";
    /// Check for raycasts and if they detect any object in their path.
    pub const RAYCAST_DETECTION : &str = "phy_raycast_detection";
}

impl Plugin for Physics2dPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let settings = self.settings.clone();

        // Stage order goes as follows
        // Joints step -> Physics step -> collision detection -> solve -> sync -> Raycast detection  

        app
            .insert_resource(settings)
            .add_stage_before(CoreStage::Update, stage::PHYSICS_STEP, SystemStage::single_threaded())
            .add_stage_before(stage::PHYSICS_STEP, stage::JOINT_STEP,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_STEP, stage::SENSOR_RESET_STEP,SystemStage::single_threaded())
            .add_stage_after(stage::SENSOR_RESET_STEP, stage::COLLISION_DETECTION, SystemStage::parallel())
            .add_stage_after(stage::COLLISION_DETECTION, stage::PHYSICS_SOLVE,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_SOLVE, stage::RAYCAST_DETECTION, SystemStage::single_threaded());

        // Add the event type
        app.add_event::<AABBCollisionEvent>();
        app.add_event::<CollisionEvent>();

        // Add the systems themselves for each step
        app.add_system_to_stage(stage::PHYSICS_STEP, physics_step_system.system())
            .add_system_to_stage(stage::COLLISION_DETECTION, shape_coll_take_2::<Square,Square>.system())
            // .add_system_to_stage(stage::COLLISION_DETECTION, shape_coll_take_2::<Circle,Circle>.system())
            .add_system_to_stage(stage::PHYSICS_SOLVE, aabb_solve_system.system())
            .add_system_to_stage(stage::RAYCAST_DETECTION, raycast_system.system());
        // TODO Recreate the Joint systems

    }
}

fn get_child_shapes(shapes : &Query<&Aabb>, children : &Children) -> Option<Aabb> {
    for &e in children.iter() {
        if let Ok(shape) = shapes.get_component::<Aabb>(e) {
            return Some(*shape);
        }
    }
    None
}

/// Checks for collision between 2 AABB objects and returns the penetration(of a in b) if existing
fn get_aabb_collision(a : Aabb, b : Aabb) -> bool {
    let amin = a.position - a.extents;
    let amax = a.position + a.extents;
    let bmin = b.position - b.extents;
    let bmax = b.position + b.extents;

    // Check for a general collision
    let coll_x = amax.x >= bmin.x && bmax.x >= amin.x;
    let coll_y = amax.y >= bmin.y && bmax.y >= amin.y;

    coll_x && coll_y
}

type WithOr<S,T> = Or<(With<S>,With<T>)>;

#[allow(clippy::clippy::too_many_arguments, clippy::type_complexity)]
fn shape_coll_take_2<S,T> (
    phy_sets : Res<PhysicsSettings>, // Physics Settings - general thing we(usually) need
    // General Queries
    mut transforms : Query<&mut GlobalTransform, WithOr<S,T>>,
    mut kinematic_bodies : Query<&mut KinematicBody2D, WithOr<S,T>>,
    mut _sensors : Query<&mut Sensor2D, With<T>>,
    // Entity based Queries
    kinematic_entities_s : Query<(Entity, &S), (With<KinematicBody2D>, With<GlobalTransform>)>,
    _kinematic_entities_t : Query<(Entity, &T), With<(KinematicBody2D, GlobalTransform)>>,
    static_entities : Query<(Entity, &T, &StaticBody2D), With<GlobalTransform>>,
    _sensor_entities : Query<(Entity, &T), With<(Sensor2D, GlobalTransform)>>,
    // Event writer
    mut writer : EventWriter<CollisionEvent>,
) where
    S : Shape + Component,
    T : Shape + Component,
{
    let trans_mode = phy_sets.transform_mode;

    // Loop over entities, borrow them immutually, get the data we need and calculate everything later

    for (a_entity, a_shape) in kinematic_entities_s.iter() {        
        // Get basic data(like aabb and position)
        let a_kin = match kinematic_bodies.get_component::<KinematicBody2D>(a_entity) {
            Ok(b) => b,
            Err(_) => continue,
        };
        // they are out of the loop because we will emit only 1 collision event(because there is not reason to emit all of them)
        let mut remainder = Vec2::ZERO;
        let mut normal = Vec2::ZERO;
        let (a_trans, movement) = {
            let gt = transforms.get_component::<GlobalTransform>(a_entity);
            let mut t : Transform2D = match gt {
                Ok(gt) => (gt,trans_mode).into(),
                Err(_) => continue,
            };
            let movement = t.translation - a_kin.prev_position;
            t.translation = a_kin.prev_position;

            (t,movement)
        };
        let movement_len = movement.length();
        let move_normalized = movement.normalize();
        let a_aabb = a_shape.to_aabb(a_trans);
        let move_extents = (a_aabb.extents * movement.signum()).project(move_normalized);
        let a_aabb_move = a_shape.to_aabb_move(movement + move_extents, a_trans);

        // Compare to static bodies
        for (static_entity, static_shape, static_body) in static_entities.iter() {
            // check for layer-mask collision
            if ((a_kin.mask & static_body.layer) | (a_kin.layer & static_body.mask)) == 0 {
                continue;
            }

            let static_trans = match transforms.get_component::<GlobalTransform>(static_entity) {
                Ok(t) => Transform2D::from((t, trans_mode)),
                Err(_) => continue,
            };

            // Check for the general aabb collision
            let static_aabb = static_shape.to_aabb(static_trans);
            if get_aabb_collision(a_aabb_move, static_aabb) {
                // get the normal static aabb for a simple ray v aabb test

                let res = solve_raycast_aabb(
                    a_trans.translation,
                    move_normalized,
                    static_aabb,
                );
                if res >= 0.0 {
                    let coll_move_part = if res > movement_len { movement_len } else { res };

                    let coll_pos = move_normalized * coll_move_part + a_trans.translation - move_extents;
                    
                    let a_trans_coll = Transform2D {
                        translation : coll_pos,
                        ..a_trans
                    };
                    let (dis, is_pen) = a_shape.collide_with_shape(
                        a_trans_coll,
                        static_shape,
                        static_trans
                    );
                    if is_pen { // on collision basically
                        normal = dis.normalize(); // this is flipped
                        let angle_cos = move_normalized.dot(normal);
                        remainder = if angle_cos.abs() < 0.5 {
                            dis // They are perpendicular, there is no triangle...
                        }
                        else {
                            // c = a / cos(alpha) - cosin definition...
                            dis.length() * angle_cos.recip() * move_normalized
                        };
                        
                        let new_position = a_trans.translation + movement + remainder;
                        // move the kinematic body to a "safe" place
                        match transforms.get_component_mut::<GlobalTransform>(a_entity) {
                            Ok(mut t) => {
                                println!("pen moving kin to {}, rem {} movement {} origin {} dis {}",
                                new_position,
                                remainder,
                                movement,
                                a_trans.translation,
                                dis
                            );
                                trans_mode.set_position(&mut t, new_position);
                            },
                            Err(_) => continue,
                        }
                        // normal and remainder are flipped so...
                        normal = -normal;
                        remainder = -remainder;
                    }
                    else if dis == Vec2::ZERO {
                        let vertex = move_normalized * (coll_move_part + 0.1) + a_trans.translation;
                        let (norm, _) = static_shape.get_vertex_penetration(vertex, static_trans);
                        normal = norm.normalize(); 
                    }
                    else {
                        normal = -(dis.normalize());

                        // Get the remainder amount
                        let angle_cos = move_normalized.dot(normal);

                        remainder = if angle_cos.abs() < 0.5 {
                            dis // They are perpendicular, there is no triangle...
                        }
                        else {
                            // c = a / cos(alpha) - cosin definition...
                            -dis.length() * angle_cos.recip() * move_normalized
                        };
                        if remainder.project(move_normalized).length_squared() > movement_len.powi(2) {
                            remainder = Vec2::ZERO;
                        }

                        let new_pos = a_trans.translation + movement - remainder;

                        match transforms.get_component_mut::<GlobalTransform>(a_entity) {
                            Ok(mut t) => {
                                println!("else moving kin to {}", new_pos);
                                trans_mode.set_position(&mut t, new_pos);
                            },
                            Err(_) => continue,
                        }
                        if remainder == Vec2::ZERO {
                            normal = Vec2::ZERO;
                        }

                    }
                    
                }
            }
        }
        // println!("position {} vel {}", a_trans.translation, a_kin.linvel);
        if normal != Vec2::ZERO {
            // println!("normal {} reminader {}", normal, remainder);
            writer.send(CollisionEvent {
                entity_a: a_entity,
                entity_b: a_entity,
                with_static: false,
                normal,
                remainder,
            });

            match kinematic_bodies.get_component_mut::<KinematicBody2D>(a_entity) {
                Ok(mut k) => {
                    let vel = k.linvel.slide(normal);
                    k.linvel = vel;
                },
                Err(_) => continue,
            }
        }
    }
}

// TODO Remove this(or technically replace?)
fn aabb_solve_system (
    mut collisions : EventReader<AABBCollisionEvent>,
    mut bodies : Query<&mut KinematicBody2D>,
    mut transforms : Query<&mut GlobalTransform>,
    staticbodies : Query<&StaticBody2D>,
    phys_set : Res<PhysicsSettings>,
) {
    let trans_mode = phys_set.transform_mode;
    let mut add_position = move |entity : Entity, amount : Vec2 | {
        if let Ok(mut t) = transforms.get_component_mut::<GlobalTransform>(entity) {
            let new_pos = trans_mode.get_position(&t) + amount;
            trans_mode.set_position(&mut t, new_pos);
        }
    };


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
            check_on_stuff(&mut a, normal, &phys_set);

            // if colliding with a static object, just undo the penetration and slide across the normal(aka pen direction)
            // TODO Maybe add a step functionality here?

            if a.linvel.signum() != coll.penetration.signum() {
                let project = a.linvel.project(normal);
                let slide = a.linvel - project; // This is pretty much how slide works

                let linvel = slide - project * with_sb.bounciness.max(a.bounciness) * a.stiffness;

                a.linvel = linvel;
                // Update position
                add_position(coll.entity_a, coll.penetration);
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

            // a & b refrences are no longer valid!
            match bodies.get_component_mut::<KinematicBody2D>(coll.entity_b) {
                Ok(mut b) => {
                    let stiff = b.stiffness;
                    b.dynamic_acc -= impulse * stiff;
                    check_on_stuff(&mut b, -normal, &phys_set);

                    if b.linvel.signum() != -coll.penetration.signum() {
                        b.linvel = b.linvel.slide(normal);
                        // Update the position
                        add_position(coll.entity_b, -coll.penetration);
                    }
                },
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_b);
                    continue;
                }
            };
            match bodies.get_component_mut::<KinematicBody2D>(coll.entity_a) {
                Ok(mut a) => {
                    let stiff = a.stiffness;
                    a.dynamic_acc += impulse * stiff;
                    check_on_stuff(&mut a, -normal, &phys_set);
                    if a.linvel.signum() != coll.penetration.signum() {
                        a.linvel = a.linvel.slide(normal);
                        // Update position
                        add_position(coll.entity_a, coll.penetration);
                    }
                    check_on_stuff(&mut a, normal, &phys_set);
                },
                Err(_) => {
                    eprintln!("Couldnt get KinematicBody2D of entity {:?}", coll.entity_a);
                    continue;
                }
            };
        }
    }
}
/// Checks for `on_floor`,`on_wall`,`on_ceil`
fn check_on_stuff(body : &mut KinematicBody2D, normal : Vec2, phy_set : &PhysicsSettings) {
    let angle = phy_set.floor_angle;
    let up = -phy_set.gravity.normalize();
    let dot = up.dot(normal);

    if dot >= angle {
        body.on_floor = Some(normal);
    }
    if dot.abs() < angle {
        body.on_wall = Some(normal);
    }
    if dot <= -angle {
        body.on_ceil = Some(normal);
    }
}

/// apply gravity, movement, rotation, forces, friction and other stuff as well
fn physics_step_system (
    time : Res<Time>,
    physics_sets : Res<PhysicsSettings>,
    mut query : Query<(&mut KinematicBody2D, &mut GlobalTransform)>,
) {
    let delta = time.delta_seconds();
    let gravity = physics_sets.gravity;
    let trans_mode = physics_sets.transform_mode;

    for (mut body, mut transform) in query.iter_mut() {
        if !body.active {
            continue;
        }
        body.prev_position = trans_mode.get_position(&transform);

        let accelerating = body.accumulator.length_squared() > 0.1 || body.dynamic_acc.length_squared() > 0.1;

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
        let position = trans_mode.get_position(&transform) + body.linvel * delta;
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

fn raycast_system(
    phy_set : Res<PhysicsSettings>,
    mut query : Query<(&mut RayCast2D, &GlobalTransform)>,
    kinematics : Query<(Entity, &KinematicBody2D, &GlobalTransform, &Children)>,
    statics : Query<(Entity, &StaticBody2D, &GlobalTransform, &Children)>,
    shapes : Query<&Aabb>
) {
    let trans_mode = phy_set.transform_mode;

    for (mut ray, global_transform) in query.iter_mut() {
        let pos = trans_mode.get_position(&global_transform) + ray.offset;

        let mut closest = 1.1f32;
        let mut closest_entity = None;
        let mut is_static = false;

        for (entity, kin, kin_trans, children) in kinematics.iter() {
            // Handle collision layers please thank you
            if (kin.layer & ray.mask) == 0 {
                continue;
            } 

            // get the collider
            let collider = match get_child_shapes(&shapes, &children) {
                Some(c) => c,
                None => continue,
            };
            
            let kin_pos = trans_mode.get_position(&kin_trans);

            let coll = collide_ray_aabb(pos, ray.cast, collider, kin_pos);

            if let Some(f) = coll {
                if f < closest && f > 0.0 {
                    closest = f;
                    closest_entity = Some(entity);
                }
            }
        }
        if ray.collide_with_static {
            for (entity, stc, stc_trans, children) in statics.iter() {
                if (stc.layer & ray.mask) == 0 {
                    continue;
                }
                // Get the collider
                let collider = match get_child_shapes(&shapes, &children) {
                    Some(c) => c,
                    None => continue,
                };

                let stc_pos = trans_mode.get_position(&stc_trans);

                let coll = collide_ray_aabb(pos, ray.cast, collider, stc_pos);

                if let Some(f) = coll {
                    if f < closest && f > 0.0 {
                        closest = f;
                        closest_entity = Some(entity);
                        is_static = true;
                    }
                }
            }
        }

        // Combine all the calculations into 1 big pile of stuff
        if closest > 0.0 && closest <= 1.0 {
            if let Some(ce) = closest_entity {
                let collision_pos = pos + closest * ray.cast;

                let coll = RayCastCollision {
                    collision_point : collision_pos,
                    entity : ce, // closest_entity is surely some if we reach this piece of code
                    is_static,
                };
                ray.collision = Some(coll);
            }
            else {
                ray.collision = None;
            }
        }
        else {
            ray.collision = None;
        }
    }
}

/// This method returns a value betweem [0.0, inf] which can be used as
///
/// `collision_point = ray_from + result * ray_to`
///
/// Do note that a collision happened across the ray only if `0 < result <= 1`
pub fn collide_ray_aabb(
    ray_from : Vec2, 
    ray_cast : Vec2, 
    aabb : Aabb, 
    aabb_pos : Vec2
) -> Option<f32> {
    // How this works?
    //      https://gdbooks.gitbooks.io/3dcollisions/content/Chapter3/raycast_aabb.html
    // tl;dr:
    //      We treat the ray as a line, and solve for when the line intersects with the sides of the box basically

    let aabb_min = aabb_pos - aabb.extents;
    let aabb_max = aabb_pos + aabb.extents;


    // if one of the cast components is 0.0, make sure we are in the bounds of that axle
    // Why?
    //      We do this explicit check because the raycast formula i used doesnt handle cases where one of the components is 0
    //       as it would lead to division by 0(thus errors) and the `else NAN` part will make it completly ignore the collision
    //       on that axle
    if ray_cast.x == 0.0 {
        let ray_min = ray_from.x.min(ray_from.x + ray_cast.x);
        let ray_max = ray_from.x.max(ray_from.x + ray_cast.x);

        if !(aabb_min.x <= ray_max && aabb_max.x >= ray_min) {
            return None; // if it doesnt collide on the X axle terminate it early
        }
    }
    if ray_cast.y == 0.0 {
        let ray_min = ray_from.y.min(ray_from.y + ray_cast.y);
        let ray_max = ray_from.y.max(ray_from.y + ray_cast.y);

        if !(aabb_min.y <= ray_max && aabb_max.y >= ray_min) {
            return None; // if it doesnt collide on the X axle terminate it early
        }
    }

    // The if else's are to make sure we dont divide by 0.0, because if the ray is parallel to one of the axis
    // it will never collide(thus division by 0.0)
    let xmin = if ray_cast.x != 0.0 { (aabb_min.x - ray_from.x) / ray_cast.x } else { f32::NAN };
    let xmax = if ray_cast.x != 0.0 { (aabb_max.x - ray_from.x) / ray_cast.x } else { f32::NAN };
    let ymin = if ray_cast.y != 0.0 { (aabb_min.y - ray_from.y) / ray_cast.y } else { f32::NAN };
    let ymax = if ray_cast.y != 0.0 { (aabb_max.y - ray_from.y) / ray_cast.y } else { f32::NAN };
    
    let min = (xmin.min(xmax)).max(ymin.min(ymax));
    let max = (xmin.max(xmax)).min(ymin.max(ymax));

    if max < 0.0 || min > max {
        None
    }
    else if min < 0.0 {
        Some(max)
    }
    else {
        Some(min)
    }
}

/// Dont use this, use collide_ray_aabb instead... how did you even got this?
fn solve_raycast_aabb(
    ray_from : Vec2,
    ray_cast : Vec2,
    aabb : Aabb,
) -> f32 {
    let aabb_min = aabb.position - aabb.extents;
    let aabb_max = aabb.position + aabb.extents;

    // if one of the cast components is 0.0, make sure we are in the bounds of that axle
    // Why?
    //      We do this explicit check because the raycast formula i used doesnt handle cases where one of the components is 0
    //       as it would lead to division by 0(thus errors) and the `else NAN` part will make it completly ignore the collision
    //       on that axle
    if ray_cast.x == 0.0 {
        let ray_min = ray_from.x.min(ray_from.x + ray_cast.x);
        let ray_max = ray_from.x.max(ray_from.x + ray_cast.x);

        if !(aabb_min.x <= ray_max && aabb_max.x >= ray_min) {
            return -1.0; // if it doesnt collide on the X axle terminate it early
        }
    }
    if ray_cast.y == 0.0 {
        let ray_min = ray_from.y.min(ray_from.y + ray_cast.y);
        let ray_max = ray_from.y.max(ray_from.y + ray_cast.y);

        if !(aabb_min.y <= ray_max && aabb_max.y >= ray_min) {
            return -1.0; // if it doesnt collide on the X axle terminate it early
        }
    }

    // The if else's are to make sure we dont divide by 0.0, because if the ray is parallel to one of the axis
    // it will never collide(thus division by 0.0)
    let xmin = if ray_cast.x != 0.0 { (aabb_min.x - ray_from.x) / ray_cast.x } else { f32::NAN };
    let xmax = if ray_cast.x != 0.0 { (aabb_max.x - ray_from.x) / ray_cast.x } else { f32::NAN };
    let ymin = if ray_cast.y != 0.0 { (aabb_min.y - ray_from.y) / ray_cast.y } else { f32::NAN };
    let ymax = if ray_cast.y != 0.0 { (aabb_max.y - ray_from.y) / ray_cast.y } else { f32::NAN };
    
    let min = (xmin.min(xmax)).max(ymin.min(ymax));
    let max = (xmin.max(xmax)).min(ymin.max(ymax));

    if max < 0.0 {
        -1.0
    }
    else if min > max {
        max
    }
    else if min < 0.0 {
        max
    }
    else {
        min
    }
}