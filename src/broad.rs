use crate::{
    bodies::*,
    physics_components::velocity::Vel,
    transform_mode::TransformMode,
    shapes::*,
};
use bevy::prelude::*;

/// Kinematic body's entity(with vels) with its surrounding static bodies(without vels)
pub struct BroadData {
    /// Kinematic entity
    pub(crate) entity : Entity,
    pub(crate) inst_vel : Vec2,
    /// Static bodies in the area(who wants to chat)
    pub(crate) area : Vec<Entity>,
    /// Sensors in the area(dont trip the alarm!)
    pub(crate) sensors : Vec<Entity>,
}
/// Kinematic body pairs, which might collide during broad phase calculation
pub struct KinematicCollisionCouple {
    pub(crate) a : Entity,
    pub(crate) b : Entity,
}
// TODO make broad_phase_2 for kinematic collision couples

/// Simply pushes ObbData and ObbDataKinematic into the event system for every shape
pub fn broad_phase_1(
    time : Res<Time>,
    trans_mode : Res<TransformMode>,
    kinematics : Query<(Entity, &CollisionShape, Option<&Vel>, &GlobalTransform),With<Vel>>,
    statics : Query<(Entity, &CollisionShape, &GlobalTransform),(Without<Vel>, Without<Sensor2D>)>,
    sensors : Query<(Entity, &CollisionShape, &GlobalTransform), With<Sensor2D>>,
    mut broad_writer : EventWriter<BroadData>,
) {
    // TODO Optimize it later, when all is done and the earth is gone
    // probably get space partition or quad trees up and running

    // TODO check for layer/mask!!!

    let delta = time.delta_seconds();

    for (e, cs,  vel, gt) in kinematics.iter() {
        let inst_vel = vel.unwrap_or(&Vel::ZERO).0 * delta;

        let aabb = cs.shape().to_aabb((gt, *trans_mode).into());

        let circle_center = aabb.position;
        let circle_radius_sqrd = (inst_vel + aabb.extents).length_squared();

        // Get all staticbodies which might collide with use
        let mut st_en : Vec<Entity> = Vec::new();
        for (se, scs, sgt) in statics.iter() {
            let saabb = scs.shape().to_aabb((*trans_mode, sgt).into());

            if aabb_circle(
                circle_center,
                circle_radius_sqrd,
                &saabb,
            ) {
                st_en.push(se);
            }
        }
        // same for sensors(we do the extra calculations for sensors which does not move)
        let mut se_en : Vec<Entity> = Vec::new();
        for (se, scs, sgt) in sensors.iter() {
            let saabb = scs.shape().to_aabb((*trans_mode, sgt).into());


            if aabb_circle(
                circle_center,
                circle_radius_sqrd,
                &saabb,
            ) {
                se_en.push(se);
            }
        }
        // wrap it up to an event
        broad_writer.send(BroadData {
            entity : e,
            inst_vel,
            area : st_en,
            sensors : se_en,
        });
    }
}

fn aabb_circle(
    center : Vec2,
    radius_sqrd : f32,
    aabb : &Aabb,
) -> bool {
    let min = aabb.position - aabb.extents;
    let max = aabb.position + aabb.extents;

    let distance = min.max(center.min(max)) - center;

    distance.length_squared() < radius_sqrd
}
