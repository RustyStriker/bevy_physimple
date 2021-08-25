use crate::{
    bodies::*, 
    broad::ConBroadData, 
    physics_components::{
        Transform2D, 
        Vel
    }, 
    plugin::CollisionEvent, 
    prelude::VecOp, 
    shapes::*,
};
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn narrow_phase_system(
    shapes : Query<&CollisionShape>,
    mut vels : Query<&mut Vel>,
    mut transforms : Query<&mut Transform2D>,
    mut sensors : Query<&mut Sensor>,
    mut broad_data : EventReader<ConBroadData>,
    // Writer to throw collision events
    mut collision_writer : EventWriter<CollisionEvent>,
) {
    // Loop over kinematic bodies
    // Capture their sensor/static surroundings
    // Move all kinematic bodies to where they need to be moved
    // check collision pairs between kinematic bodies

    // We need to transfer it into a Vec(or other iterable stuff) because the EventReader.iter is a 1 time consuming thingy
    let broad_data = broad_data.iter().collect::<Vec<_>>();

    for &broad in broad_data.iter() {
        let entity_kin = broad.entity;

        // TODO normal error messages would be better i guess?
        let mut kin_trans = match transforms.get_component::<Transform2D>(entity_kin) {
            Ok(t) => t.clone(),
            Err(_) => continue,
        };

        let kin_shape = match shapes.get(entity_kin) {
            Ok(s) => s,
            Err(_) => continue, // Add debug stuff
        };
        let shape_kin = kin_shape.shape();

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

            for (se, _) in broad.area.iter() {
                let cmove = movement - remainder; // Basically only the movement left without the "recorded" collisions
                let cmove_ray = (cmove.normalize(), cmove.length());

                // Get the obb shape thingy
                let s_shape = match shapes.get(*se) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let s_shape = s_shape.shape();

                let s_transform = match transforms.get_component::<Transform2D>(*se) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let coll_position = s_shape.collide_ray(s_transform, cmove_ray, kin_trans.translation());
                let coll_position = coll_position.unwrap_or(cmove_ray.1);

                let coll_pos = Transform2D::new(
                    kin_trans.translation() + cmove_ray.0 * coll_position,
                    kin_trans.rotation(),
                    kin_trans.scale()
                );

                let dis = shape_kin.collide(&coll_pos, s_shape, s_transform);
                let dis2 = s_shape.collide(s_transform, shape_kin, &coll_pos);

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
                    let new_pos = coll_pos.translation() + dis;
                    normal = dis.normalize();

                    let moved = new_pos - kin_trans.translation();
                    remainder = movement - moved;

                    coll_entity = Some(*se);
                }
                
            } // out of the surroindings for loop
            // We gonna check here for sensors, as we dont want to include it in our "main loop"
            // and we want to check only when we know exactly how much we go further to avoid ghost triggers
            for (se, _) in broad.sensors.iter() { // SENSOR LOOP!!!!
                // this was pretty mostly copied from above
                let cmove = movement - remainder; // Basically only the movement left without the "recorded" collisions
                let cmove_ray = (cmove.normalize(), cmove.length());

                // Get the obb shape thingy
                let s_shape = match shapes.get(*se) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let s_shape = s_shape.shape();

                let s_transform = match transforms.get_component::<Transform2D>(*se) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let coll_position = s_shape.collide_ray(s_transform, cmove_ray, kin_trans.translation());
                let coll_position = coll_position.unwrap_or(cmove_ray.1);

                let coll_pos = Transform2D::new(
                    kin_trans.translation() + cmove_ray.0 * coll_position,
                    kin_trans.rotation(),
                    kin_trans.scale()
                );

                let dis = shape_kin.collide(&coll_pos, s_shape, s_transform);
                let dis2 = s_shape.collide(s_transform, shape_kin, &coll_pos);

                // we dont really care how far we are penetrating, only that we indeed are penetrating
                if dis.is_some() || dis2.is_some() {
                    // we indeed collide
                    if let Ok(mut sensor) = sensors.get_mut(*se) {
                        if !sensor.bodies.contains(&entity_kin) {
                            sensor.bodies.push(entity_kin);
                        }
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
                kin_trans.add_translation(movement - remainder);

                let rem_proj = remainder.project(normal);
                let rem_slide = remainder - rem_proj;

                // basically what we still need to move
                movement = rem_slide; // same thing as 147
                                      // - rem_proj * staticbody.bounciness.max(kin.bounciness) * kin.stiffness;


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
                kin_trans.add_translation(movement); // need to move whatever left to move with
                break;
            }
        } // out of loop(line 94)

        // We cloned the body's Transform2D to avoid mutability issues, so now we reapply it
        if let Ok(mut t) = transforms.get_mut(entity_kin) {
            *t = kin_trans;
        }
    } // out of kin_obb for loop
}