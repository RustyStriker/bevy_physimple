use bevy::{math::Mat2, prelude::*};
use crate::{physics_components::Transform2D, prelude::*};

pub struct CollPairKin(Entity, Entity);
pub struct CollPairStatic(Entity, Entity);
pub struct CollPairSensor(Entity, Entity);

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn broad_phase_2(
	shapes: Query<&CollisionShape>,
	// bodies
	kins: Query<(Entity, &Transform2D, &CollisionLayer),(Without<Vel>, Without<StaticBody>, Without<Sensor>)>,
	kins_con: Query<(Entity, &Transform2D, &CollisionLayer), With<Vel>>,
	statics: Query<(Entity, &Transform2D, &CollisionLayer),With<StaticBody>>,
	sensors: Query<(Entity, &Transform2D, &CollisionLayer), With<Sensor>>,
	// event writers
	mut pair_kin: EventWriter<CollPairKin>,
	mut pair_static: EventWriter<CollPairStatic>,
	mut pair_sensor: EventWriter<CollPairSensor>,
) {
	// Someday this function should utilize the different algorithms and data strucs
	// to make for a better broad phase with superiour performance

	// Current imlp is for something that just works, without too much hassle

	// Kinematic_con x kinematic_con
	for (i, (e1, t1, l1)) in kins_con.iter().enumerate() {
		let aabb1 = match shapes.get(e1) {
			Ok(s) => s.aabb(t1),
			Err(_) => continue,
		};

		for (e2, t2, l2) in kins_con.iter().skip(i + 1) {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_kin.send(CollPairKin(e1,e2));
				}
			}
		}
	}

	// Kinematic x _
	for (i, (e1, t1, l1)) in kins.iter().enumerate() {
		let aabb1 = match shapes.get(e1) {
			Ok(s) => s.aabb(t1),
			Err(_) => continue,
		};

		// x Kinematic
		for (e2, t2, l2) in kins.iter().skip(i + 1) {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_kin.send(CollPairKin(e1,e2));
				}
			}
		}

		// x Kinematic_con
		for (e2, t2, l2) in kins_con.iter() {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_kin.send(CollPairKin(e1,e2));
				}
			}
		}

		// x Statics
		for (e2, t2, l2) in statics.iter() {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_static.send(CollPairStatic(e1,e2));
				}
			}
		}

		// x Sensors
		for (e2, t2, l2) in sensors.iter() {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_sensor.send(CollPairSensor(e1,e2));
				}
			}
		}
	}

}
#[allow(clippy::too_many_arguments)]
pub fn narrow_phase_2(
	// Data we need
	shapes: Query<&CollisionShape>,
	mut transforms: Query<&mut Transform2D>,
	mut sensors: Query<&mut Sensor>,
	mut vels: Query<&mut Vel>,
	// Readers(for the entities)
	mut pair_kin: EventReader<CollPairKin>,
	mut pair_static: EventReader<CollPairStatic>,
	mut pair_sensor: EventReader<CollPairSensor>,
	// writers
	mut coll_writer: EventWriter<CollisionEvent>,
) {
	// Solve kinematic pairs
	for CollPairKin(e1, e2) in pair_kin.iter() {
		let s1 = match shapes.get(*e1) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let t1 = match transforms.get_component::<Transform2D>(*e1) {
			Ok(t) => t,
			Err(_) => continue,
		};
		
		let s2 = match shapes.get(*e2) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let t2 = match transforms.get_component::<Transform2D>(*e2) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let p = collide(s1,t1,s2,t2);

		if let Some(pen) = p {
			let normal = pen.normalize();

			coll_writer.send(CollisionEvent { 
				entity_a: *e1, 
				entity_b: *e2, 
				is_b_static: false, 
				normal,
			});
			// Maybe move both of them? or should i just move 1 of them?
			// I also cannot tell which 1 is moving here, so that's a bummer
			// for now i will move only e1
			if let Ok(mut t) = transforms.get_mut(*e1) {
				t.add_translation(pen);
			}

			// slide the movement of the objects
			if let Ok(mut v) = vels.get_mut(*e1) {
				if v.0.dot(normal) < 0.0 {
					v.0 = v.0.slide(normal);
				}
			}
			if let Ok(mut v) = vels.get_mut(*e2) {
				if v.0.dot(-normal) < 0.0 {
					v.0 = v.0.slide(normal);
				}
			}
		}

	}

	// Solve static pairs
	for CollPairStatic(ek, es) in pair_static.iter() {
		let sk = match shapes.get(*ek) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let tk = match transforms.get_component::<Transform2D>(*ek) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let ss = match shapes.get(*es) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let ts = match transforms.get_component::<Transform2D>(*es) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let p = collide(sk,tk,ss,ts);

		if let Some(pen) = p {
			coll_writer.send(CollisionEvent{
				entity_a: *ek,
				entity_b: *es,
				is_b_static: true,
				normal: pen.normalize(),
			});

			if let Ok(mut t) = transforms.get_mut(*ek) {
				t.add_translation(pen);
			}
		}
	}

	// "Solve" sensor pairs
	for CollPairSensor(ek, es) in pair_sensor.iter() {
		let sk = match shapes.get(*ek) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let tk = match transforms.get_component::<Transform2D>(*ek) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let ss = match shapes.get(*es) {
			Ok(s) => s,
			Err(_) => continue,
		};

		let ts = match transforms.get_component::<Transform2D>(*es) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let p = collide(sk,tk,ss,ts);

		if p.is_some() {
			if let Ok(mut sen) = sensors.get_mut(*es) {
				if !sen.bodies.contains(ek) {
					sen.bodies.push(*ek);
				}
			}
		}
	}
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ray_phase(
	trans: Query<&Transform2D>,
	layers: Query<&CollisionLayer>,
	mut rays: Query<(Entity, &mut RayCast)>,
	kins: Query<(Entity, &CollisionShape),(Without<StaticBody>, Without<Sensor>)>,
	stts: Query<(Entity, &CollisionShape),With<StaticBody>>,
) {
	for (re, mut r) in rays.iter_mut() {
		let rl = match layers.get(re) {
			Ok(l) => l,
			Err(_) => continue,
		};

		let rt = match trans.get(re) {
			Ok(t) => t,
			Err(_) => continue,
		};

		if r.collide_with_static {
			let bodies_iter = kins.iter()
				.chain(stts.iter())
				.filter(|(e, ..)| layers.get(*e).unwrap_or(&CollisionLayer::ZERO).overlap(rl))
				// Make sure everyone have a transform
				.filter(|(e,..)| trans.get(*e).is_ok()) 
				.map(|(e, c)| (e, c, trans.get(e).unwrap()));
			
			r.collision = collide_ray(&r, rt, bodies_iter);
		}
		else {
			let bodies_iter = kins.iter()
				.filter(|(e, ..)| layers.get(*e).unwrap_or(&CollisionLayer::ZERO).overlap(rl))
				// Make sure everyone have a transform
				.filter(|(e,..)| trans.get(*e).is_ok()) 
				.map(|(e, c)| (e, c, trans.get(e).unwrap()));
			
			r.collision = collide_ray(&r, rt, bodies_iter);
		}
	}
}

/// # collide_ray
/// 
/// This function allows you to conjure an iterator where `Item = (Entity, &CollisionShape, &Transform2D)`
/// eg. every iterator which returns `(Entity, &CollisionShape, &Transform2D)`
/// 
/// ## A Couple of notes
/// 
/// - This function does not check for layer/mask collision, so it is completely possible to attempt collision
/// with a shape which normally wouldn't collide with the ray due to different collision layers.
/// 
/// - You need to provide it with `Transform2D`, so it you changed the transform of the entity
/// after the `Transform2D->Transform` sync point, its own `Transform2D` woulnd't update, and so its `GlobalTransform`,
/// thus you may experience a 1 frame delay/bugs due to checking an exact frame
/// (generally it is better to work between the sync points and with `Transform2D` instead of `Transform` so everything will
/// stay updated during physics calculations)
/// 
pub fn collide_ray<'a,T>(
	ray: &RayCast,
	ray_trans: &Transform2D,
	bodies: T,
) -> Option<RayCastCollision> 
where
	T: Iterator<Item = (Entity, &'a CollisionShape, &'a Transform2D)>
{
	let r_rot = Mat2::from_angle(ray_trans.rotation());
	let r_cast = r_rot * ray.cast;
	let r_origin = ray_trans.translation() + r_rot * ray.offset;

	let mut shortest = f32::INFINITY;
	let mut short_entity = None;

	// Collide over kins
	for (be,bs, bt) in bodies {
		// TODO add aabb testing or something else first
		
		let c = bs.ray(bt, r_origin, r_cast);
		
		if let Some(c) = c {
			if c > 0.0 && c < 1.0 && c < shortest {
				shortest = c;
				short_entity = Some(be);
			}
		}
	}

	short_entity.map(|e| RayCastCollision {
		collision_point: shortest * r_cast + r_origin,
		entity: e,
		is_static: false,
	})
}