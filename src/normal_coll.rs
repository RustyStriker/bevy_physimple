use bevy::prelude::*;
use crate::{physics_components::Transform2D, prelude::*};

pub struct CollPairKin(Entity, Entity);
pub struct CollPairStatic(Entity, Entity);
pub struct CollPairSensor(Entity, Entity);

pub fn broad_phase_2(
	shapes : Query<&CollisionShape>,
	// bodies
	kins : Query<(Entity, &Transform2D, &CollisionLayer),(Without<Vel>, Without<StaticBody>, Without<Sensor>)>,
	kins_con : Query<(Entity, &Transform2D, &CollisionLayer), With<Vel>>,
	statics : Query<(Entity, &Transform2D, &CollisionLayer),With<StaticBody>>,
	sensors : Query<(Entity, &Transform2D, &CollisionLayer), With<Sensor>>,
	// event writers
	mut pair_kin : EventWriter<CollPairKin>,
	mut pair_static : EventWriter<CollPairStatic>,
	mut pair_sensor : EventWriter<CollPairSensor>,
) {
	// Someday this function should utilize the different algorithms and data strucs
	// to make for a better broad phase with superiour performance

	// Current imlp is for something that just works, without too much hassle

	// Kinematic_con x kinematic_con
	for (i, (e1, t1, l1)) in kins_con.iter().enumerate() {
		let aabb1 = match shapes.get(e1) {
			Ok(s) => s.shape().to_aabb(t1),
			Err(_) => continue,
		};

		for (e2, t2, l2) in kins_con.iter().skip(i + 1) {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.shape().to_aabb(t2),
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
			Ok(s) => s.shape().to_aabb(t1),
			Err(_) => continue,
		};

		// x Kinematic
		for (e2, t2, l2) in kins.iter().skip(i + 1) {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.shape().to_aabb(t2),
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
					Ok(s) => s.shape().to_aabb(t2),
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
					Ok(s) => s.shape().to_aabb(t2),
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
					Ok(s) => s.shape().to_aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					pair_sensor.send(CollPairSensor(e1,e2));
				}
			}
		}
	}

}

pub fn narrow_phase_2(
	// Data we need
	shapes : Query<&CollisionShape>,
	mut transforms : Query<&mut Transform2D>,
	mut sensors : Query<&mut Sensor>,
	mut vels : Query<&mut Vel>,
	// Readers(for the entities)
	mut pair_kin : EventReader<CollPairKin>,
	mut pair_static : EventReader<CollPairStatic>,
	mut pair_sensor : EventReader<CollPairSensor>,
	// writers
	mut coll_writer : EventWriter<CollisionEvent>,
) {
	// Solve kinematic pairs
	for CollPairKin(e1, e2) in pair_kin.iter() {
		let s1 = match shapes.get(*e1) {
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let t1 = match transforms.get_component::<Transform2D>(*e1) {
			Ok(t) => t,
			Err(_) => continue,
		};
		
		let s2 = match shapes.get(*e2) {
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let t2 = match transforms.get_component::<Transform2D>(*e2) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let p1 = s1.collide(t1, s2, t2);
		let p2 = s2.collide(t2, s1, t1);

		if let Some(pen) = match (p1, p2) {
			(Some(p1), Some(p2)) => {
				if p1.length_squared() < p2.length_squared() {
					Some(p1)
				}
				else {
					Some(-p2)
				}
			},
			(Some(p1), None) => Some(p1),
			(None, Some(p2)) => Some(-p2),
			(None, None) => None, // pretty much ignore
		} {
			let normal = pen.normalize();

			coll_writer.send(CollisionEvent { 
				entity_a: *e1, 
				entity_b: *e2, 
				is_b_static: false, 
				normal: normal,
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
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let tk = match transforms.get_component::<Transform2D>(*ek) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let ss = match shapes.get(*es) {
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let ts = match transforms.get_component::<Transform2D>(*es) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let pen1 = sk.collide(tk, ss, ts);
		let pen2 = ss.collide(ts, sk, tk);

		if let Some(pen) = match (pen1, pen2) {
			(Some(p1), Some(p2)) => {
				if p1.length_squared() < p2.length_squared() {
					Some(p1)
				}
				else {
					Some(-p2)
				}
			},
			(Some(p1), None) => Some(p1),
			(None, Some(p2)) => Some(-p2),
			(None, None) => None, // pretty much ignore
		} {
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
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let tk = match transforms.get_component::<Transform2D>(*ek) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let ss = match shapes.get(*es) {
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let ts = match transforms.get_component::<Transform2D>(*es) {
			Ok(t) => t,
			Err(_) => continue,
		};

		let pen1 = sk.collide(tk, ss, ts);
		let pen2 = ss.collide(ts, sk, tk);

		if pen1.is_some() || pen2.is_some() {
			if let Ok(mut sen) = sensors.get_mut(*es) {
				if !sen.bodies.contains(ek) {
					sen.bodies.push(*ek);
				}
			}
		}
	}
}