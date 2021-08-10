use crate::{bodies::*, physics_components::{angular_velocity::AngVel, velocity::Vel}, settings::TransformMode, shapes::*};
use bevy::prelude::*;

/// Kinematic body's entity(with vels) with its surrounding static bodies(without vels)
pub struct BroadData {
    /// Kinematic entity
    pub(crate) entity : Entity,
    pub(crate) inst_vel : Vec2,
    /// Static bodies in the area(who wants to chat)
    pub(crate) area : Vec<Entity>,
    /// Sensors in the area(dont trip the alarm!)
    pub(crate) sensors : Vec<Entity>, // TODO check for sensors in broad
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
    kinematics : Query<(Entity, &Obv, Option<&Vel>, &GlobalTransform), Or<(With<Vel>, With<AngVel>)>>,
    statics : Query<(Entity, &Obv, &GlobalTransform), (Without<Vel>, Without<AngVel>)>,
    sensors : Query<(Entity, &Obv, &GlobalTransform), With<Sensor2D>>,
    mut broad_writer : EventWriter<BroadData>,
) {
    // TODO Optimize it later, when all is done and the earth is gone
    // probably get space partition or quad trees up and running

    let delta = time.delta_seconds();

    for (e, obv, vel, gt) in kinematics.iter() {
        
        let inst_vel = vel.unwrap_or(&Vel::ZERO).0 * delta;

        let circle_center = trans_mode.get_global_position(gt) + obv.offset;
        let circle_radius_sqrd = (inst_vel + get_obv_extents(obv)).length_squared();
        
        // Get all staticbodies which might collide with use
        let mut st_en : Vec<Entity> = Vec::new();
        for (se, sv, sgt) in statics.iter() {
            if obv_circle(circle_center, circle_radius_sqrd, sv, trans_mode.get_global_position(sgt)) {
                st_en.push(se);
            }
        }
        // same for sensors(we do the extra calculations for sensors which does not move)
        let mut se_en : Vec<Entity> = Vec::new();
        for (se, sv, sgt) in sensors.iter() {
            if obv_circle(circle_center, circle_radius_sqrd, sv, trans_mode.get_global_position(sgt)) {
                se_en.push(se);
            }
        }
        // wrap it up to an event
        broad_writer.send(BroadData {
            entity: e,
            inst_vel,
            area: st_en,
            sensors: se_en,
        });
        
    }
}

fn obv_circle(
    center : Vec2,
    radius_sqrd : f32,
    obv : &Obv,
    obv_pos : Vec2,
) -> bool {
    let obv_pos = obv_pos + obv.offset;

    match &obv.shape {
        BoundingShape::Aabb(b) => {
            let min = obv_pos - b.extents;
            let max = obv_pos + b.extents;

            let distance = min.max(center.min(max)) - center;

            distance.length_squared() < radius_sqrd
        },
        BoundingShape::Circle(c) => {
            let distance = center - obv_pos;

            // This is a crude way of doing it based on the formula `(a-b)^2 <= a^2 + b^2`
            distance.length_squared() < c.radius.powi(2) + radius_sqrd
        }
    }
}

fn get_obv_extents(obv : &Obv) -> Vec2 {
    match &obv.shape {
        BoundingShape::Aabb(a) => a.extents,
        BoundingShape::Circle(c) => Vec2::splat(c.radius),
    }
}
