//! This module provides the primitives and systems for 2d physics simulation.
//!
//! For examples, see the root of the crate.

use bevy::math::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::broad::{self, BoundingBox, Collider};
use crate::common::*;
use crate::bodies::*;
use crate::shapes::*;

/// This is what you want to add to your `App` if you want to run 2d physics simulation.
pub struct Physics2dPlugin; // {
    // friction : Vec2,
    // gravity : Vec2,
    // step : f32,
// }

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
        app
            .insert_resource(GlobalFriction::default())
            .insert_resource(GlobalGravity::default())
            .insert_resource(TranslationMode::default())
            .insert_resource(RotationMode::default())
            .insert_resource(GlobalStep::default())
            .insert_resource(GlobalUp::default())
            .insert_resource(AngularTolerance::default())
            .add_stage_before(CoreStage::Update, stage::PHYSICS_STEP, SystemStage::single_threaded())
            .add_stage_before(stage::PHYSICS_STEP, stage::COLLIDING_JOINT,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_STEP, stage::BROAD_PHASE,SystemStage::single_threaded())
            .add_stage_after(stage::BROAD_PHASE, stage::NARROW_PHASE,SystemStage::single_threaded())
            .add_stage_after(stage::NARROW_PHASE, stage::PHYSICS_SOLVE,SystemStage::single_threaded())
            .add_stage_after(stage::PHYSICS_SOLVE, stage::RIGID_JOINT,SystemStage::single_threaded())
            .add_stage_after(stage::RIGID_JOINT, stage::SYNC_TRANSFORM,SystemStage::single_threaded());

        // Add the event type
        app.add_event::<AABBCollisionEvent>();


        app.add_system_to_stage(stage::PHYSICS_STEP, physics_step_system_2.system())
            .add_system_to_stage(stage::NARROW_PHASE, aabb_collision_detection_system.system())
            .add_system_to_stage(stage::PHYSICS_SOLVE, aabb_solve_system.system())
            .add_system_to_stage(stage::SYNC_TRANSFORM, sync_transform_system_2.system())
            .add_system_to_stage(
                FixedJointBehaviour::STAGE,
                joint_system::<FixedJointBehaviour>.system(),
            )
            .add_system_to_stage(
                MechanicalJointBehaviour::STAGE,
                joint_system::<MechanicalJointBehaviour>.system(),
            )
            .add_system_to_stage(
                SpringJointBehaviour::STAGE,
                joint_system::<SpringJointBehaviour>.system(),
            );
    }
}

pub type BroadPhase = broad::BroadPhase<Obb>;

/// The global gravity that affects every `RigidBody` with the `Semikinematic` status.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlobalGravity(pub Vec2);

/// The global step value, affects all semikinematic bodies.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlobalStep(pub f32);

/// The global up vector, affects all semikinematic bodies.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlobalUp(pub Vec2);

/// The global angular tolerance in radians, affects all semikinematic bodies.
///
/// This is used for step calculation and for push dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngularTolerance(pub f32);

impl Default for AngularTolerance {
    fn default() -> Self {
        Self(30.0_f32.to_radians())
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obb {
    status: Status,
    body: Entity,
    position: Vec2,
    rotation: Mat2,
    vertices: [Vec2; 4],
    normals: [Vec2; 4],
}

impl Obb {
    fn new(
        status: Status,
        body: Entity,
        rotation: Mat2,
        position: Vec2,
        v0: Vec2,
        v1: Vec2,
        v2: Vec2,
        v3: Vec2,
        n0: Vec2,
        n1: Vec2,
    ) -> Self {
        Self {
            status,
            body,
            rotation,
            position,
            vertices: [v0, v1, v2, v3],
            normals: [-n1, n0, n1, -n0],
        }
    }

    pub fn v0(&self) -> Vec2 {
        self.rotation * self.vertices[0] + self.position
    }

    pub fn v1(&self) -> Vec2 {
        self.rotation * self.vertices[1] + self.position
    }

    pub fn v2(&self) -> Vec2 {
        self.rotation * self.vertices[2] + self.position
    }

    pub fn v3(&self) -> Vec2 {
        self.rotation * self.vertices[3] + self.position
    }

    pub fn min(&self) -> Vec2 {
        let min_x = self
            .v0()
            .x
            .min(self.v1().x)
            .min(self.v2().x)
            .min(self.v3().x);
        let min_y = self
            .v0()
            .y
            .min(self.v1().y)
            .min(self.v2().y)
            .min(self.v3().y);
        Vec2::new(min_x, min_y)
    }

    pub fn max(&self) -> Vec2 {
        let max_x = self
            .v0()
            .x
            .max(self.v1().x)
            .max(self.v2().x)
            .max(self.v3().x);
        let max_y = self
            .v0()
            .y
            .max(self.v1().y)
            .max(self.v2().y)
            .max(self.v3().y);
        Vec2::new(max_x, max_y)
    }

    pub fn get_support(&self, dir: Vec2) -> Vec2 {
        let mut best_projection = f32::MIN;
        let mut best_vertex = Vec2::ZERO;

        for i in 0..4 {
            let v = self.vertices[i];
            let proj = v.dot(dir);

            if proj > best_projection {
                best_vertex = v;
                best_projection = proj;
            }
        }

        best_vertex
    }
}

impl Collider for Obb {
    type Point = Vec2;

    fn bounding_box(&self) -> BoundingBox<Self::Point> {
        BoundingBox::new(self.min(), self.max())
    }

    fn status(&self) -> Status {
        self.status
    }
}

/// The two dimensional size of a `Shape`
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Size2 {
    pub width: f32,
    pub height: f32,
}

impl Size2 {
    /// Returns a new 2d size.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// The shape of a rigid body.
///
/// Contains a rotation/translation offset and a size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Shape {
    offset: Vec2,
    size: Size2,
}

impl Shape {
    /// Return a new `Shape` with a zero offset and a size.
    pub fn new(size: Size2) -> Self {
        let offset = Vec2::ZERO;
        Self { offset, size }
    }

    /// Return a new `Shape` with an offset and a size.
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}

impl From<Size2> for Shape {
    fn from(size: Size2) -> Self {
        let x = size.width * 0.5;
        let y = size.height * 0.5;
        let offset = Vec2::new(-x, -y);
        Self { offset, size }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InnerJoint {
    body1: Entity,
    body2: Entity,
    offset: Vec2,
    angle: f32,
}

impl InnerJoint {
    pub fn new(body1: Entity, body2: Entity) -> Self {
        Self {
            body1,
            body2,
            offset: Vec2::ZERO,
            angle: 0.0,
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }
}

/// Defines a set of behaviours on how joints should move the anchored body relative to the anchor.
pub trait JointBehaviour: Send + Sync + 'static {
    const STAGE: &'static str = stage::COLLIDING_JOINT;

    /// Returns a new position for target based on `self` and `anchor`.
    fn position(
        &mut self,
        _offset: Vec2,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<Vec2> {
        None
    }

    /// Returns a new rotation for target based on `self` and `anchor`.
    fn rotation(&mut self, _angle: f32, _anchor: &RigidBody, _target: &RigidBody) -> Option<f32> {
        None
    }

    /// Returns a new linear velocity for target based on `self` and `anchor`.
    fn linear_velocity(
        &mut self,
        _offset: Vec2,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<Vec2> {
        None
    }

    /// Returns a new angular velocity for target based on `self` and `anchor`.
    fn angular_velocity(
        &mut self,
        _angle: f32,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<f32> {
        None
    }

    /// Returns a linear impulse to apply to target based on `self` and `anchor`.
    fn linear_impulse(
        &mut self,
        _offset: Vec2,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<Vec2> {
        None
    }

    /// Returns an angular impulse to apply to target based on `self` and `anchor`.
    fn angular_impulse(
        &mut self,
        _angle: f32,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<f32> {
        None
    }
}

/// A joint behaviour that causes the anchored body to be rigidly fixed at an offset and an angle.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct FixedJointBehaviour;

impl JointBehaviour for FixedJointBehaviour {
    const STAGE: &'static str = stage::RIGID_JOINT;

    fn position(&mut self, offset: Vec2, anchor: &RigidBody, _target: &RigidBody) -> Option<Vec2> {
        Some(anchor.position + offset)
    }

    fn rotation(&mut self, angle: f32, anchor: &RigidBody, _target: &RigidBody) -> Option<f32> {
        Some(anchor.rotation + angle)
    }
}

/// A joint behaviour that causes the anchored body to be accurately positioned with an offset and an angle.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MechanicalJointBehaviour;

impl JointBehaviour for MechanicalJointBehaviour {
    const STAGE: &'static str = stage::COLLIDING_JOINT;

    fn position(&mut self, offset: Vec2, anchor: &RigidBody, _target: &RigidBody) -> Option<Vec2> {
        Some(anchor.position + offset)
    }

    fn rotation(&mut self, angle: f32, anchor: &RigidBody, _target: &RigidBody) -> Option<f32> {
        Some(anchor.rotation + angle)
    }

    fn linear_velocity(
        &mut self,
        _offset: Vec2,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<Vec2> {
        Some(Vec2::ZERO)
    }

    fn angular_velocity(
        &mut self,
        _angle: f32,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<f32> {
        Some(0.0)
    }
}

/// A joint behaviour that will move the anchored body into a position and angle relative to the anchor over time.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct SpringJointBehaviour {
    rigidness: f32,
}

impl SpringJointBehaviour {
    /// Create a new SpringJointBehaviour with an exact rigidness value.
    ///
    /// Rigidness describes how "snappy" the spring joint is. When it's at 0.0,
    /// the anchored body will "jump" into position softly over one second.
    /// When it's at 1.0, the anchored body will "jump" into position almost instantaenously.
    pub fn new(rigidness: f32) -> Option<SpringJointBehaviour> {
        if rigidness < 0.0 || rigidness >= 1.0 {
            None
        } else {
            Some(Self { rigidness })
        }
    }

    pub fn new_lossy(rigidness: f32) -> SpringJointBehaviour {
        Self {
            rigidness: rigidness.max(0.0).min(1.0),
        }
    }
}

impl JointBehaviour for SpringJointBehaviour {
    const STAGE: &'static str = stage::COLLIDING_JOINT;

    fn linear_velocity(
        &mut self,
        _offset: Vec2,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<Vec2> {
        Some(Vec2::ZERO)
    }

    fn angular_velocity(
        &mut self,
        _angle: f32,
        _anchor: &RigidBody,
        _target: &RigidBody,
    ) -> Option<f32> {
        Some(0.0)
    }

    fn linear_impulse(
        &mut self,
        offset: Vec2,
        anchor: &RigidBody,
        target: &RigidBody,
    ) -> Option<Vec2> {
        // the minimum time to "jump" into position
        const EPSILON: f32 = 0.1;
        // the maximum time to "jump" into position
        const T: f32 = 1.0;
        let springiness = 1.0 - self.rigidness;
        let position = anchor.position + offset;
        let d = position - target.position;
        let scale = (T - EPSILON) * springiness + EPSILON;
        let impulse = d * target.mass / scale;
        Some(impulse)
    }

    fn angular_impulse(
        &mut self,
        angle: f32,
        anchor: &RigidBody,
        target: &RigidBody,
    ) -> Option<f32> {
        // the minimum time to "jump" into position
        const EPSILON: f32 = 0.1;
        // the maximum time to "jump" into position
        const T: f32 = 1.0;
        let springiness = 1.0 - self.rigidness;
        let rotation = anchor.rotation + angle;
        let d = rotation - target.rotation;
        let scale = (T - EPSILON) * springiness + EPSILON;
        let impulse = d * target.mass / scale;
        Some(impulse)
    }
}

/// Allows one `RigidBody` to be anchored at another one
/// in a fixed way, along with a local offset and angle.
pub type FixedJoint = Joint<FixedJointBehaviour>;

/// Allows one `RigidBody` to be anchored at another one
/// in a physically accurate way, along with a local offset and angle.
pub type MechanicalJoint = Joint<MechanicalJointBehaviour>;

/// Allows one `RigidBody` to be anchored at another one
/// in a spring-y way, along with a local offset and angle.
pub type SpringJoint = Joint<SpringJointBehaviour>;

impl SpringJoint {
    /// Add a rigidness value to an owned `Joint`.
    pub fn with_rigidness(mut self, rigidness: f32) -> Self {
        self.behaviour.rigidness = rigidness;
        self
    }
}

/// Allows one `RigidBody` to be anchored at another one
/// in a pre-defined way, along with a local offset and angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Joint<B: JointBehaviour> {
    inner: InnerJoint,
    behaviour: B,
}

impl<B: JointBehaviour + Default> Joint<B> {
    /// Create a new joint, where the second body shall be anchored at the first body.
    pub fn new(body1: Entity, body2: Entity) -> Self {
        Self::with_behaviour(body1, body2, B::default())
    }
}

impl<B: JointBehaviour> Joint<B> {
    /// Create a new joint, where the second body shall be anchored at the first body.
    pub fn with_behaviour(body1: Entity, body2: Entity, behaviour: B) -> Self {
        Self {
            inner: InnerJoint::new(body1, body2),
            behaviour,
        }
    }

    /// Add an offset to an owned `Joint`.
    pub fn with_offset(self, offset: Vec2) -> Self {
        Self {
            inner: self.inner.with_offset(offset),
            behaviour: self.behaviour,
        }
    }

    /// Add an angle to an owned `Joint`.
    pub fn with_angle(self, angle: f32) -> Self {
        Self {
            inner: self.inner.with_angle(angle),
            behaviour: self.behaviour,
        }
    }
}

/// The rigid body.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct RigidBody {
    /// Current position of this rigid body.
    pub position: Vec2,
    lowest_position: Vec2,
    /// Current rotation of this rigid body.
    ///
    /// NOTE: collisions checks may or may not be broken if this is not a multiple of 90 degrees.
    pub rotation: f32,
    /// Current linear velocity of this rigid body.
    pub linvel: Vec2,
    prev_linvel: Vec2,
    /// The terminal linear velocity of a semikinematic body.
    ///
    /// Defaults to `f32::INFINITY`.
    pub terminal: Vec2,
    accumulator: Vec2,
    dynamic_acc: Vec2,
    /// Current angular velocity of this rigid body.
    pub angvel: f32,
    prev_angvel: f32,
    /// The terminal angular velocity of a semikinematic body.
    ///
    /// Defaults to `f32::INFINITY`.
    pub ang_term: f32,
    /// The status, i.e. static or semikinematic.
    ///
    /// Affects how forces and collisions affect this rigid body.
    pub status: Status,
    mass: f32,
    inv_mass: f32,
    active: bool,
    sensor: bool,

    // wether the body is touching a surface
    on_floor : Option<Vec2>,
    on_wall : Option<Vec2>,
    on_ceil : Option<Vec2>,
}

impl RigidBody {
    /// Returns a new `RigidBody` with just a mass and all other components set to their defaults.
    pub fn new(mass: Mass) -> Self {
        Self {
            position: Vec2::ZERO,
            lowest_position: Vec2::ZERO,
            rotation: 0.0,
            linvel: Vec2::ZERO,
            prev_linvel: Vec2::ZERO,
            terminal: Vec2::new(f32::INFINITY, f32::INFINITY),
            accumulator: Vec2::ZERO,
            dynamic_acc: Vec2::ZERO,
            angvel: 0.0,
            prev_angvel: 0.0,
            ang_term: f32::INFINITY,
            status: Status::Semikinematic,
            mass: mass.scalar(),
            inv_mass: mass.inverse(),
            active: true,
            sensor: false,
            on_floor : None,
            on_wall : None,
            on_ceil : None,
        }
    }

    /// Returns a `RigidBody` identical to this one, but with the position set to a new one.
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the rotation set to a new one.
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the linear velocity set to a new one.
    pub fn with_linear_velocity(mut self, linvel: Vec2) -> Self {
        self.linvel = linvel;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the linear velocity set to a new one.
    pub fn with_angular_velocity(mut self, angvel: f32) -> Self {
        self.angvel = angvel;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the terminal linear velocity set to a new one.
    pub fn with_terminal(mut self, terminal: Vec2) -> Self {
        self.terminal = terminal;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the terminal linear velocity set to a new one.
    pub fn with_angular_terminal(mut self, terminal: f32) -> Self {
        self.ang_term = terminal;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the acceleration set to a new one.
    pub fn with_acceleration(mut self, acceleration: Vec2) -> Self {
        self.accumulator = acceleration;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the status set to a new one.
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the active flag set to a new one.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Returns a `RigidBody` identical to this one, but with the sensor flag set to a new one.
    pub fn with_sensor(mut self, sensor: bool) -> Self {
        self.sensor = sensor;
        self
    }

    /// Applies an impulse to the `RigidBody`s linear velocity.
    pub fn apply_linear_impulse(&mut self, impulse: Vec2) {
        self.linvel += impulse * self.inv_mass;
    }

    /// Applies an impulse to the `RigidBody`s linear velocity.
    pub fn apply_angular_impulse(&mut self, impulse: f32) {
        self.angvel += impulse * self.inv_mass;
    }

    /// Applies a force to the `RigidBody`s acceleration accumulator.
    pub fn apply_force(&mut self, force: Vec2) {
        self.accumulator += force * self.inv_mass;
    }

    /// Gets the active flag.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Gets the sensor flag.
    pub fn is_sensor(&self) -> bool {
        self.sensor
    }

    /// Gets the mass
    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// Gets the mass
    pub fn inverse_mass(&self) -> f32 {
        self.inv_mass
    }

    /// Sets the active flag.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Sets the sensor flag.
    pub fn set_sensor(&mut self, sensor: bool) {
        self.sensor = sensor;
    }

    /// Sets the mass.
    pub fn set_mass(&mut self, mass: Mass) {
        self.mass = mass.scalar();
        self.inv_mass = mass.inverse();
    }

    /// Returns the difference between the last known linear velocity and the current linear velocity.
    pub fn linear_deceleration(&self) -> Vec2 {
        self.prev_linvel.abs() - self.linvel.abs()
    }

    /// Returns the difference between the last known angular velocity and the current angular velocity.
    pub fn angular_deceleration(&self) -> f32 {
        self.prev_angvel.abs() - self.angvel.abs()
    }

    /// Get Floor normal if body is on floor
    pub fn on_floor(&self) -> Option<Vec2> {
        self.on_floor
    }
    /// Get wall normal if body is touching a wall
    pub fn on_wall(&self) -> Option<Vec2> {
        self.on_wall
    }
    /// Get ceilling normal if body is touching a ceiling
    pub fn on_ceil(&self) -> Option<Vec2> {
        self.on_ceil
    }
}

/// The manifold, representing detailed data on a collision between two `RigidBody`s.
///
/// Usable as an event.
#[derive(Debug, Clone)]
pub struct Manifold {
    /// The first entity.
    pub body1: Entity,
    /// The second entity.
    pub body2: Entity,
    /// The penetration, relative to the second entity.
    pub penetration: f32,
    /// The normal, relative to the second entity.
    pub normal: Vec2,
    /// The contact points of this manifold.
    pub contacts: SmallVec<[Vec2; 4]>,
}

pub fn broad_phase_system(
    mut commands: Commands,
    query: Query<(Entity, &RigidBody, &Children)>,
    query2: Query<&Shape>,
) {
    let mut colliders = Vec::new();
    for (entity, body, children) in &mut query.iter() {
        for &e in children.iter() {
            if let Ok(shape) = query2.get_component::<Shape>(e) {
                let v0 = shape.offset;
                let v1 = shape.offset + Vec2::new(shape.size.width, 0.0);
                let v2 = shape.offset + Vec2::new(shape.size.width, shape.size.height);
                let v3 = shape.offset + Vec2::new(0.0, shape.size.height);
                let rotation = Mat2::from_angle(body.rotation);
                let position = body.position;
                let n0 = Vec2::new(1.0, 0.0);
                let n1 = Vec2::new(0.0, 1.0);
                let collider = Obb::new(
                    body.status,
                    entity,
                    rotation,
                    position,
                    v0,
                    v1,
                    v2,
                    v3,
                    n0,
                    n1,
                );
                colliders.push(collider);
            }
        }
    }
    let broad = BroadPhase::with_colliders(colliders);
    commands.insert_resource(broad);
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
    
) {
    for coll in collisions.iter() {
        let mut a = bodies.get_component_mut::<KinematicBody2D>(coll.entity_a).unwrap();
        
        let normal = coll.penetration.normalize();

        // Check for floor/wall/ceil collision(maybe change it later to only static bodies?)
        // TODO Switch to user defined values
        const FLOOR_ANGLE : f32 = 0.7;
        let up = Vec2::new(0.0,1.0);
        let dot = up.dot(normal);

        if dot >= FLOOR_ANGLE {
            a.on_floor = Some(normal);
        }
        if dot.abs() < FLOOR_ANGLE {
            a.on_wall = Some(normal);
        }
        if dot <= -FLOOR_ANGLE {
            a.on_ceil = Some(normal);
        }

        if coll.with_static {
            // if colliding with a static object, just undo the penetration and slide across the normal(aka pen direction)
            // TODO Maybe add a step functionality here?

            if a.linvel.signum() != coll.penetration.signum() {
                a.linvel = a.linvel.slide(normal);
                a.position += coll.penetration;
            }
        }
    }
}

/// apply gravity, movement, rotation, forces, friction and other stuff as well
fn physics_step_system_2 (
    time : Res<Time>,
    friction : Res<GlobalFriction>,
    gravity : Res<GlobalGravity>,
    mut query : Query<&mut KinematicBody2D>,
) {
    let delta = time.delta_seconds();

    for mut body in query.iter_mut() {
        if !body.active {
            continue;
        }

        // Gravity
        if body.mass > f32::EPSILON {
            body.linvel += gravity.0 * delta;
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
        // TODO better friciton based on gravity orientation please
        body.linvel.x *= friction.0;
        body.angvel *= friction.0;

        // Reset on_* variables
        body.on_floor = None;
        body.on_wall = None;
        body.on_ceil = None;
    }
}

pub fn joint_system<B: JointBehaviour>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Joint<B>)>,
    mut bodies: Query<&mut RigidBody>,
) {
    for (e, mut joint) in query.iter_mut() {
        let anchor = if let Ok(anchor) = bodies.get_component::<RigidBody>(joint.inner.body1) {
            anchor
        } else {
            commands.entity(e).despawn_recursive();
            continue;
        };
        let target = if let Ok(target) = bodies.get_component::<RigidBody>(joint.inner.body2) {
            target
        } else {
            commands.entity(e).despawn_recursive();
            continue;
        };
        let offset = joint.inner.offset;
        let angle = joint.inner.angle;
        let position = joint.behaviour.position(offset, &anchor, &target);
        let rotation = joint.behaviour.rotation(angle, &anchor, &target);
        let linvel = joint.behaviour.linear_velocity(offset, &anchor, &target);
        let angvel = joint.behaviour.angular_velocity(angle, &anchor, &target);
        let linimp = joint.behaviour.linear_impulse(offset, &anchor, &target);
        let angimp = joint.behaviour.angular_impulse(angle, &anchor, &target);

        let mut target = bodies
            .get_component_mut::<RigidBody>(joint.inner.body2)
            .unwrap();

        if let Some(position) = position {
            target.position = position;
        }

        if let Some(rotation) = rotation {
            target.rotation = rotation;
        }

        if let Some(linvel) = linvel {
            target.linvel = linvel;
        }

        if let Some(angvel) = angvel {
            target.angvel = angvel;
        }

        if let Some(linimp) = linimp {
            target.apply_linear_impulse(linimp);
        }

        if let Some(angimp) = angimp {
            target.apply_angular_impulse(angimp);
        }
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

pub fn sync_transform_system(
    translation_mode: Res<TranslationMode>,
    rotation_mode: Res<RotationMode>,
    mut query : Query<(&RigidBody, &mut Transform)>,
) {
    for (body, mut transform) in query.iter_mut() {
        match *translation_mode {
            TranslationMode::AxesXY => {
                let x = body.position.x;
                let y = body.position.y;
                let z = 0.0;
                transform.translation = Vec3::new(x, y, z);
            }
            TranslationMode::AxesXZ => {
                let x = body.position.x;
                let y = 0.0;
                let z = body.position.y;
                transform.translation = Vec3::new(x, y, z);
            }
            TranslationMode::AxesYZ => {
                let x = 0.0;
                let y = body.position.x;
                let z = body.position.y;
                transform.translation = Vec3::new(x, y, z);
            }
        }
        match *rotation_mode {
            RotationMode::AxisX => {
                transform.rotation = Quat::from_rotation_x(body.rotation);
            }
            RotationMode::AxisY => {
                transform.rotation = Quat::from_rotation_y(body.rotation);
            }
            RotationMode::AxisZ => {
                transform.rotation = Quat::from_rotation_z(body.rotation);
            }
        }
    }
}

pub fn sync_transform_system_2 (
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