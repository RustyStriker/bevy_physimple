use crate::{
    bodies::*, broad::BroadData, physics_components::velocity::Vel, plugin::CollisionEvent,
    prelude::VecOp, shapes::*,
};
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn narrow_phase_system(
    // Shape queries
    shapes : Query<&CollisionShape>,
    obvs : Query<&Obv>,
    // The different bodies
    mut kinematics : Query<&mut KinematicBody2D>,
    mut vels : Query<&mut Vel>,
    global_transforms : Query<&GlobalTransform>,
    mut transforms : Query<&mut Transform>,
    mut sensors : Query<&mut Sensor2D>,
    // Simple collision data
    mut broad_data : EventReader<BroadData>,
    // Writer to throw collision events
    mut collision_writer : EventWriter<CollisionEvent>,
) {
    // Loop over kinematic bodies
    // Capture their sensor/static surroundings
    // Move all kinematic bodies to where they need to be moved
    // check collision pairs between kinematic bodies

    // We need to transfer it into a Vec(or other iterable stuff) because the EventReader.iter is a 1 time consuming thingy
    let broad_data = broad_data.iter().collect::<Vec<_>>();

    let trans_mode = crate::settings::TransformMode::XY;
    let up_dir = Vec2::Y;

    for broad in broad_data.iter() {
        let entity_kin = broad.entity;

        let mut kin = match kinematics.get_component_mut::<KinematicBody2D>(entity_kin) {
            Ok(k) => k,
            Err(_) => {
                eprintln!(
                    "Entity {} is missing a kinematic body(how did you get here? >_>)",
                    entity_kin.id()
                );
                continue;
            }
        };

        // TODO normal error messages would be better i guess?
        let mut kin_pos = match global_transforms.get_component::<GlobalTransform>(entity_kin) {
            Ok(t) => Transform2D::from((t, trans_mode)),
            Err(_) => continue,
        };

        let shape_kin = match shapes.get(entity_kin) {
            Ok(s) => s,
            Err(_) => continue, // Add debug stuff
        };
        let shape_kin = shape_kin.shape();

        let mut iter_amount = 5; // Maximum number of collision detection - should probably be configureable
        let mut movement = broad.inst_vel; // Current movement to check for

        loop {
            if iter_amount == 0 {
                break;
            }
            iter_amount -= 1;

            let mut normal = Vec2::ZERO;
            let mut remainder = Vec2::ZERO;
            let mut coll_entity : Option<Entity> = None;

            for se in broad.area.iter() {
                let cmove = movement - remainder; // Basically only the movement left without the "recorded" collisions

                let s_obv = match obvs.get(*se) {
                    Ok(o) => o,
                    Err(_) => {
                        continue;
                    }
                };

                let s_transform = match global_transforms.get(*se) {
                    Ok(t) => Transform2D::from((t, trans_mode)),
                    Err(_) => continue,
                };

                let coll_position =
                    raycast_obv(kin_pos.translation, cmove, s_obv, s_transform.translation);
                let coll_position = coll_position.min(1.0); // Lock coll_position between [0,1]

                if (coll_position + 1.0).abs() >= f32::EPSILON {
                    // coll_position != -1
                    // Get the obb shape thingy
                    let s_shape = match shapes.get(*se) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let s_shape = s_shape.shape();

                    // we should probably do it using the actual shape, since it might not work properly for with bounding volume
                    let coll_pos = Transform2D {
                        translation : kin_pos.translation + cmove * coll_position,
                        ..kin_pos
                    };

                    let dis = shape_kin.collide(coll_pos, s_shape, s_transform);
                    let dis2 = s_shape.collide(s_transform, shape_kin, coll_pos);

                    // if we use dis2 we need to reverse the direction
                    let dis = if let Some(d1) = dis {
                        if let Some(d2) = dis2 {
                            if d1.length_squared() < d2.length_squared() {
                                Some(d1)
                            }
                            else {
                                Some(-d2)
                            }
                        }
                        else {
                            dis
                        }
                    }
                    else {
                        dis2.map(|d| -d)
                    };

                    if let Some(dis) = dis {
                        let new_pos = coll_pos.translation + dis;
                        normal = dis.normalize();

                        let moved = new_pos - kin_pos.translation;
                        remainder = movement - moved;

                        coll_entity = Some(*se);
                    }
                }
            } // out of the surroindings for loop

            // We gonna check here for sensors, as we dont want to include it in our "main loop"
            // and we want to check only when we know exactly how much we go further to avoid ghost triggers
            for se in broad.sensors.iter() {
                // this was pretty mostly copied from above
                let cmove = movement - remainder; // Basically only the movement left without the "recorded" collisions

                let s_obv = match obvs.get(*se) {
                    Ok(o) => o,
                    Err(_) => {
                        continue;
                    }
                };

                let s_transform = match global_transforms.get(*se) {
                    Ok(t) => Transform2D::from((t, trans_mode)),
                    Err(_) => continue,
                };

                let coll_position =
                    raycast_obv(kin_pos.translation, cmove, s_obv, s_transform.translation);
                let coll_position = coll_position.min(1.0).max(0.0); // Lock coll_position between [0,1]

                // TODO maybe do put some sort of simpler collision assurance here?

                // Get the obb shape thingy
                let s_shape = match shapes.get(*se) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let s_shape = s_shape.shape();

                let coll_pos = Transform2D {
                    translation : kin_pos.translation + cmove * coll_position,
                    ..kin_pos
                };

                let dis = shape_kin.collide(coll_pos, s_shape, s_transform);
                let dis2 = s_shape.collide(s_transform, shape_kin, coll_pos);

                // we dont really care how far we are penetrating, only that we indeed are penetrating
                if dis.is_some() || dis2.is_some() {
                    // we indeed collide
                    if let Ok(mut sensor) = sensors.get_mut(*se) {
                        sensor.overlapping_bodies.push(entity_kin);
                    }
                    // TODO maybe also fire an event?
                }
            }

            if let Some(se) = coll_entity {
                // Supposedly to get the staticbody bounceness data
                // let staticbody = match statics.get(se) {
                //     Ok(s) => s,
                //     Err(_) => {
                //         continue;
                //     }
                // };

                // Get the vel
                let mut vel = match vels.get_mut(broad.entity) {
                    Ok(v) => v,
                    Err(_) => {
                        break;
                    }
                };

                let move_proj = vel.0.project(normal);
                let move_slide = vel.0 - move_proj;

                vel.0 = move_slide; // Redo bounciness + stiffness
                                    // - move_proj * staticbody.bounciness.max(kin.bounciness) * kin.stiffness;
                kin_pos.translation += movement - remainder;

                let rem_proj = remainder.project(normal);
                let rem_slide = remainder - rem_proj;

                // basically what we still need to move
                movement = rem_slide; // same thing as 147
                                      // - rem_proj * staticbody.bounciness.max(kin.bounciness) * kin.stiffness;

                // Do the on_* stuff
                check_on_stuff(&mut kin, normal, up_dir, 0.7);

                // Throw an event
                collision_writer.send(CollisionEvent {
                    entity_a : entity_kin,
                    entity_b : se,
                    is_b_static : true, // we only collide with static bodies here
                    normal,
                });
            }
            else {
                // There was no collisions here so we can break
                kin_pos.translation += movement; // need to move whatever left to move with
                break;
            }
        } // out of loop(line 94)

        // Set the end position of kin and its new movement

        if let Ok(mut t) = transforms.get_component_mut::<Transform>(entity_kin) {
            trans_mode.set_position(&mut t, kin_pos.translation);
        }
    } // out of kin_obb for loop
}

fn raycast_obv(
    ray_from : Vec2,
    ray_cast : Vec2,
    obv : &Obv,
    obv_pos : Vec2,
) -> f32 {
    let obv_pos = obv_pos + obv.offset;
    match &obv.shape {
        BoundingShape::Aabb(aabb) => {
            let aabb_min = obv_pos - aabb.extents;
            let aabb_max = obv_pos + aabb.extents;

            // The if else's are to make sure we dont divide by 0.0, because if the ray is parallel to one of the axis
            // it will never collide(thus division by 0.0)
            let xmin = if ray_cast.x != 0.0 {
                (aabb_min.x - ray_from.x) / ray_cast.x
            }
            else {
                f32::NAN
            };
            let xmax = if ray_cast.x != 0.0 {
                (aabb_max.x - ray_from.x) / ray_cast.x
            }
            else {
                f32::NAN
            };
            let ymin = if ray_cast.y != 0.0 {
                (aabb_min.y - ray_from.y) / ray_cast.y
            }
            else {
                f32::NAN
            };
            let ymax = if ray_cast.y != 0.0 {
                (aabb_max.y - ray_from.y) / ray_cast.y
            }
            else {
                f32::NAN
            };

            let min = (xmin.min(xmax)).max(ymin.min(ymax));
            let max = (xmin.max(xmax)).min(ymin.max(ymax));

            if max < 0.0 {
                -1.0
            }
            else if min > max || min < 0.0 {
                max
            }
            else {
                min
            }
        }
        BoundingShape::Circle(_c) => -1.0, // TODO i fucking missed that one here
    }
}

/// Checks for `on_floor`,`on_wall`,`on_ceil` - up should be normalized
fn check_on_stuff(
    body : &mut KinematicBody2D,
    normal : Vec2,
    up : Vec2,
    floor_angle : f32,
) {
    let dot = up.dot(normal);

    if dot >= floor_angle {
        body.on_floor = Some(normal);
    }
    if dot.abs() < floor_angle {
        body.on_wall = Some(normal);
    }
    if dot <= -floor_angle {
        body.on_ceil = Some(normal);
    }
}
