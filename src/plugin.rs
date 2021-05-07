use std::f32::consts::PI;

use bevy::prelude::*;

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
            .add_stage_after(stage::PHYSICS_STEP, stage::COLLISION_DETECTION,SystemStage::single_threaded())
            .add_stage_after(stage::COLLISION_DETECTION, stage::PHYSICS_SOLVE,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_SOLVE, stage::RAYCAST_DETECTION, SystemStage::single_threaded());

        // Add the event type
        app.add_event::<AABBCollisionEvent>();

        // Add the systems themselves for each step
        app.add_system_to_stage(stage::PHYSICS_STEP, physics_step_system.system())
            .add_system_to_stage(stage::COLLISION_DETECTION, aabb_collision_detection_system.system())
            .add_system_to_stage(stage::PHYSICS_SOLVE, aabb_solve_system.system())
            .add_system_to_stage(stage::RAYCAST_DETECTION, raycast_system.system());
        // TODO Recreate the Joint systems

    }
}

fn get_child_shapes(shapes : &Query<&AABB>, children : &Children) -> Option<AABB> {
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

fn aabb_collision_detection_system (
    phy_sets : Res<PhysicsSettings>,
    q_kinematic : Query<(Entity, &KinematicBody2D, &GlobalTransform, &Children)>,
    q_static : Query<(Entity, &StaticBody2D, &GlobalTransform, &Children)>,
    mut q_sensors : Query<(&mut Sensor2D, &GlobalTransform, &Children)>,
    shapes : Query<&AABB>,
    mut writer : EventWriter<AABBCollisionEvent>,
) {
    let trans_mode = phy_sets.transform_mode;

    // Clear all the sensors overlapping parts
    q_sensors.iter_mut().for_each(|(mut s,_,_)| s.overlapping_bodies.clear());
    
    let mut passed : Vec<(Entity, &KinematicBody2D, Vec2, AABB)> = Vec::new();

    // Go through all the kinematic bodies
    for (entity, body, trans, children) in q_kinematic.iter() {
        // let body_position = trans_mode.get_position()

        // Gather all the shape children(colliders...)
        let collider = match get_child_shapes(&shapes, &children) {
            Some(c) => c,
            None => continue,
        };
        
        let position = trans_mode.get_position(&trans);

        // Go through the static bodies and check for collisions
        for (se, sb, sb_trans, children) in q_static.iter() {
            // Check for masks/layers
            if (body.mask & sb.layer | body.layer & sb.mask) == 0 {
                continue;
            }

            let sc = match get_child_shapes(&shapes, &children) {
                Some(c) => c,
                None => continue,
            };

            let sb_pos = trans_mode.get_position(&sb_trans);

            // Check for collision here
            if let Some(pen) = get_aabb_collision( collider,  sc,  position,  sb_pos) {
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
        for (mut sensor, sen_trans, children) in q_sensors.iter_mut() {
            // Check for masks/layers
            if (body.mask & sensor.layer | body.layer & sensor.mask) == 0 {
                continue;
            }

            let sc = match get_child_shapes(&shapes, &children) {
                Some(c) => c,
                None => continue,
            };

            let sen_pos = trans_mode.get_position(&sen_trans);

            // Check for collision here
            if let Some(_) = get_aabb_collision(collider, sc, position, sen_pos) {
                sensor.overlapping_bodies.push(entity);
            }
        }
        // Go through all the kinematic bodies we passed already
        for (ke, ob, ob_pos, oc) in passed.iter() {
            // Check for masks/layers
            if (body.mask & ob.layer | body.layer & ob.mask) == 0 {
                continue;
            }

            // check for collisions here...
            if let Some(pen) = get_aabb_collision( collider,  *oc,  position,  *ob_pos) {
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
        
        passed.push((entity, body, position, collider));
    }
}

fn aabb_solve_system (
    mut collisions : EventReader<AABBCollisionEvent>,
    mut bodies : Query<&mut KinematicBody2D>,
    mut transforms : Query<&mut GlobalTransform>,
    staticbodies : Query<&StaticBody2D>,
    phys_set : Res<PhysicsSettings>,
) {
    let trans_mode = phys_set.transform_mode;
    let mut add_position = move |entity : Entity, amount : Vec2 | {
        match transforms.get_component_mut::<GlobalTransform>(entity) {
            Ok(mut t) => {
                let new_pos = trans_mode.get_position(&t) + amount;
                trans_mode.set_position(&mut t, new_pos);
            },
            Err(_) => { /* Maybe print an error? */},
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

            // explicit drop to convey they are not usable anymore because we borrow_mut bodies just below 
            drop(a);
            drop(b);
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
    shapes : Query<&AABB>
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
        if closest > 0.0 && closest <= 1.0 && closest_entity.is_some() {
            let collision_pos = pos + closest * ray.cast;

            let coll = RayCastCollision {
                collision_point : collision_pos,
                entity : closest_entity.unwrap(), // closest_entity is surely some if we reach this piece of code
                is_static : is_static,
            };
            ray.collision = Some(coll);
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
    aabb : AABB, 
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