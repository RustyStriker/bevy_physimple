use crate::{bodies::*, physics_components::{CollisionLayer, Transform2D, Vel}, shapes::*};
use bevy::prelude::*;

/// Kinematic body's entity(with vels) with its surrounding static bodies(without vels)
///
/// Continuous movement broad data
pub struct ConBroadData {
    /// Kinematic entity
    pub entity : Entity,
    /// Entity's aabb
    pub aabb : Aabb,
    pub inst_vel : Vec2,
    /// Static bodies in the area(who wants to chat)
    pub area : Vec<(Entity, Aabb)>,
    /// Sensors in the area(dont trip the alarm!)
    pub sensors : Vec<(Entity, Aabb)>,
}

/// Simply pushes ObbData and ObbDataKinematic into the event system for every shape
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn broad_phase_1(
    time : Res<Time>,
    kinematics : Query<(Entity, &CollisionShape, &Vel, &Transform2D, &CollisionLayer)>,
    statics : Query<(Entity, &CollisionShape, &Transform2D, &CollisionLayer),(With<StaticBody>, Without<Vel>, Without<Sensor>)>,
    sensors : Query<(Entity, &CollisionShape, &Transform2D, &CollisionLayer), With<Sensor>>,
    mut broad_writer : EventWriter<ConBroadData>,
) {
    // TODO Optimize it later, when all is done and the earth is gone
    // probably get space partition or quad trees up and running

    let delta = time.delta_seconds();

    for (e, cs,  vel, t, layer) in kinematics.iter() {
        let inst_vel = vel.0 * delta;

        let aabb = cs.aabb(t);

        let circle_center = aabb.position;
        let circle_radius_sqrd = (inst_vel + aabb.extents).length_squared();

        // Get all staticbodies which might collide with use
        let mut st_en : Vec<(Entity, Aabb)> = Vec::new();
        for (se, scs, st, sl) in statics.iter() {
            let saabb = scs.aabb(st);

            if sl.overlap(layer) && aabb_circle(
                circle_center,
                circle_radius_sqrd,
                &saabb,
            ) {
                st_en.push((se, saabb));
            }
        }
        // same for sensors(we do the extra calculations for sensors which does not move)
        let mut se_en : Vec<(Entity, Aabb)> = Vec::new();
        for (se, scs, st, sl) in sensors.iter() {
            let saabb = scs.aabb(st);


            if sl.overlap(layer) && aabb_circle(
                circle_center,
                circle_radius_sqrd,
                &saabb,
            ) {
                se_en.push((se, saabb));
            }
        }
        // wrap it up to an event
        broad_writer.send(ConBroadData {
            entity : e,
            aabb, 
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
