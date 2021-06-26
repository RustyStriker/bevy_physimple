use bevy::prelude::*;
use crate::{bodies::*, broad::{ObbData, ObbDataKinematic, ShapeType}, plugin::CollisionEvent, prelude::{PhysicsSettings, Vec2Ext}, shapes::*};

#[allow(clippy::too_many_arguments)]
pub fn narrow_phase_system(
	phy_set : Res<PhysicsSettings>,
	// Shape queries
	squares : Query<&Square>,
	circles : Query<&Circle>,
	// ... more shape queries later when i do more shapes
	// The different bodies
	mut kinematics : Query<&mut KinematicBody2D>,
	global_transforms : Query<&GlobalTransform>,
	mut transforms : Query<&mut Transform>,
	mut sensors : Query<&mut Sensor2D>,
	statics : Query<&StaticBody2D>,
	// Simple collision data
	mut obb_data : EventReader<ObbData>,
	mut obb_kinematic : EventReader<ObbDataKinematic>,
	// Writer to throw collision events
	mut collision_writer : EventWriter<CollisionEvent>,
) {
	// Loop over kinematic bodies
	// Capture their sensor/static surroundings
	// Move all kinematic bodies to where they need to be moved
	// check collision pairs between kinematic bodies

	let get_shape = |e : Entity, s : ShapeType| -> Option<&dyn Shape> {
		match s {
			ShapeType::Square => {
				match squares.get_component::<Square>(e) {
					Ok(s) => { Some(s) }
					Err(_) => { None }
				}
			}
			ShapeType::Circle => {
				match circles.get_component::<Circle>(e) {
					Ok(s) => { Some(s) }
					Err(_) => { None }
				}
			}
			ShapeType::None => {
				None
			}
		}
	};

	// We need to transfer it into a Vec(or other iterable stuff) because the EventReader.iter is a 1 time consuming thingy
	let obb_data = obb_data.iter().collect::<Vec<_>>();

	let trans_mode = phy_set.transform_mode;
	let up_dir = -phy_set.gravity.normalize();

	let mut kin_data : Vec<(Entity, &dyn Shape, Aabb)> = Vec::new();

	for obb_kin in obb_kinematic.iter() {
		let entity_kin = obb_kin.entity;
				
		let mut kin = match kinematics.get_component_mut::<KinematicBody2D>(entity_kin) {
			Ok(k) => k,
			Err(_) => {
				eprintln!("Entity {} is missing a kinematic body(how did you get here? >_>", entity_kin.id());
				continue;
			}
		};

		// TODO Maybe replace this later
		let mut kin_pos = match global_transforms.get_component::<GlobalTransform>(entity_kin) {
			Ok(t) => Transform2D::from((t, trans_mode)),
			Err(_) => continue,
		};

		let shape_kin = match get_shape(entity_kin, obb_kin.shape_type) {
			Some(s) => s,
			None => {
				eprintln!("Entity {} have no collision shape", entity_kin.id()); // TODO replace with correct log/error macro
				continue;
			}
		};
		
		// Push this body so we could check kin VS kin collisions later
		kin_data.push((entity_kin, shape_kin, obb_kin.aabb));

		// Capture stuff
		// This should be the end result of the movement
		let move_kin = kin.inst_linvel;
		
		let center = obb_kin.aabb.position;
		let radius_sqrd = (obb_kin.aabb.extents + move_kin.abs()).length_squared();
		
		let mut surroundings : Vec<&ObbData> = Vec::with_capacity(5);

		// Loop over the sensors and statics to see who we capture
		for obb in obb_data.iter() {
			let same_layer = ((obb.coll_layer & kin.mask) | (obb.coll_mask & kin.layer)) != 0;

			if same_layer && aabb_circle(center, radius_sqrd, &obb.aabb) {
				surroundings.push(obb);
			}
		}

		let mut iter_amount = 5; // Maximum number of collision detection
		// Current movement to check for
		let mut movement = move_kin;

		loop {
			if iter_amount == 0 {
				break;
			}
			iter_amount -= 1;


			let mut normal = Vec2::ZERO;
			let mut remainder = Vec2::ZERO;
			let mut coll_index = -1;

			for (i, obb) in surroundings.iter().enumerate() {
				let coll_position = raycast_aabb(kin_pos.translation, movement, obb.aabb);
				let coll_position = coll_position.min(1.0); // Lock coll_position between [0,1]

				if (coll_position + 1.0).abs() >= f32::EPSILON { // coll_position != -1
					// Get the obb shape thingy
					let obb_shape = match get_shape(obb.entity, obb.shape_type) {
						Some(s) => s,
						None => continue,
					};
					// get the obb position as well
					let obb_transform = match global_transforms.get_component::<GlobalTransform>(obb.entity) {
						Ok(t) => Transform2D::from((t,trans_mode)),
						Err(_) => continue,
					};

					let coll_pos = Transform2D {
						translation : kin_pos.translation + movement * coll_position,
						..kin_pos
					};

					let (dis, is_pen) = shape_kin.collide_with_shape(coll_pos, obb_shape, obb_transform);
					let (dis2, is_pen2) = obb_shape.collide_with_shape(obb_transform, shape_kin, coll_pos);

					let dis = if is_pen { dis } else { dis2 };
					let is_pen = is_pen | is_pen2;

					// We branch here, if the obb is a sensor we should not // TODO Maybe find a better solution to this part
					if is_pen && obb.sensor {
						match sensors.get_component_mut::<Sensor2D>(obb.entity) {
							Ok(mut s) => {
								s.overlapping_bodies.push(entity_kin);
							}
							Err(_) => { continue; }
						}
					}
					else if is_pen {
						let new_pos = coll_pos.translation + dis;
						normal = dis.normalize();
						
						let moved = new_pos - kin_pos.translation;
						remainder = movement - moved;

						// movement = movement - remainder;
						coll_index = i as i32;
					}
				}
			} // out of the surroindings for loop

			if normal != Vec2::ZERO {
				// Should we remove it? what if we are stuck between 4 walls and keep on colliding them with full bounce?
				// let obb = surroundings.remove(coll_index as usize);
				let obb = surroundings[coll_index as usize];

				if obb.sensor {
					continue; // this shouldnt happen...
				}

				let staticbody = match statics.get_component::<StaticBody2D>(obb.entity) {
					Ok(s) => s,
					Err(_) => { continue; }
				};

				let move_proj = kin.linvel.project(normal);
				let move_slide = kin.linvel - move_proj;

				kin.linvel = move_slide - move_proj * staticbody.bounciness.max(kin.bounciness) * kin.stiffness;
				kin_pos.translation += movement - remainder;

				let rem_proj = remainder.project(normal);
				let rem_slide = remainder - rem_proj;


				// basically what we still need to move
				movement = rem_slide - rem_proj * staticbody.bounciness.max(kin.bounciness) * kin.stiffness;

				// Do the on_* stuff
				check_on_stuff(&mut kin, normal, up_dir, phy_set.floor_angle);

				// Throw an event
				collision_writer.send(CollisionEvent {
					entity_a: entity_kin,
					entity_b: obb.entity,
					is_b_static: true, // we only collide with static bodies here
					normal,
				});
			}
			else { // There was no collisions here so we can break
				kin_pos.translation += movement; // need to move whatever left to move with
				break;
			}
		} // out of loop(line 94)

		// Set the end position of kin and its new movement

		if let Ok(mut t) = transforms.get_component_mut::<Transform>(entity_kin) {
			trans_mode.set_position(&mut t, kin_pos.translation);
		}
	} // out of kin_obb for loop

	// Loop over the kinematic bodies and check for collisions
	for (i, (e, s, aabb)) in kin_data.iter().enumerate() {
		let kin = match kinematics.get_component::<KinematicBody2D>(*e) {
			Ok(k) => k,
			Err(_) => continue,
		};
		
		for (e2, s2, aabb2) in kin_data.iter().skip(i + 1) {
			let kin2 = match kinematics.get_component::<KinematicBody2D>(*e2) {
				Ok(k) => k,
				Err(_) => continue,
			};

			// Skip this iteration there is no shared layer/mask thingy
			if (kin.layer & kin2.mask) | (kin.mask & kin2.layer) == 0 {
				continue;
			}

			if get_aabb_collision(*aabb, *aabb2) {
				// TODO !!!
				// let (dis, pen) = s.collide_with_shape()
			}
		}
	}
}

fn aabb_circle(center : Vec2, radius_sqrd : f32, aabb : &Aabb) -> bool {
	let aabb_min = aabb.position - aabb.extents;
	let aabb_max = aabb.position + aabb.extents;

	let distance = aabb_min.max(center.min(aabb_max)) - center;

	distance.length_squared() <= radius_sqrd
}

fn raycast_aabb(
    ray_from : Vec2,
    ray_cast : Vec2,
    aabb : Aabb,
) -> f32 {
    let aabb_min = aabb.position - aabb.extents;
    let aabb_max = aabb.position + aabb.extents;

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
    else if min > max || min < 0.0 {
        max
    }
    else {
        min
    }
}

/// Checks for `on_floor`,`on_wall`,`on_ceil` - up should be normalized
fn check_on_stuff(body : &mut KinematicBody2D, normal : Vec2, up : Vec2, floor_angle : f32) {
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

// 1. All Shapes with bodies        -> AABB + Entity + Shape Type(for later use) + Body Type(for later use as well)
// 2. AABB + ... + Kinematic        -> Capture surrounding
// 3. Captured Surroundings         -> Entity + Kinematic + Shape + Surrounding
// 4. Kinematic + ... + Surrounding -> Calculate Collisions + Solve Collisions(maybe throw some events for this)

// 1      -> Broad phase system      : Throwing events
// 2 + 3  -> Capture phase system    : Throwing events ??? 
// 4      -> Solve capture phase	 : Throw simple collision events for the user?
// 5	  -> Solve kinematic v kinematic phase