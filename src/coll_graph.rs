use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

pub struct GraphMap(pub HashMap<Entity, CollisionGraph>);
pub struct CollisionOrder(pub Vec<Entity>);

pub struct CollisionGraph {
	/// directions which the body cannot travel toward
	stuck : Vec<Vec2>,
	/// Other entities which (might) overlap the current 
	collides : Vec<Entity>,
}

pub fn kin_static_system(
	mut graphs : ResMut<GraphMap>,
	shapes : Query<&CollisionShape>,
	mut kins : Query<(Entity, &mut Transform2D, &CollisionLayer),(Without<Vel>,Without<StaticBody>,Without<Sensor>)>,
	kins_con : Query<(), With<Vel>>, // just need this to make sure entities are indeed with vel
	statics : Query<(Entity, &Transform2D, &CollisionLayer), With<StaticBody>>,
	// collision events made by the continuous collision system
	mut ccoll : EventReader<CollisionEvent>,
) {
	// clear the current graph_map
	graphs.0.clear(); // this should be trivial tbh, as the `resolve_graph_system` should consume it(alive)

	let graph = &mut graphs.0;

	// look over the kins and collide them with the statics
	for (e, mut t, l) in kins.iter_mut() {
		let mut stuck = Vec::<Vec2>::new();

		let shape = match shapes.get(e) {
			Ok(s) => s.shape(),
			Err(_) => continue,
		};

		let aabb = shape.to_aabb(&t);

		// This part can be broken into 2 systems or 2 loops, to apply actual broad phase
		// current impl just checks for aabb collision before proceeding
		for (e2, t2, l2) in statics.iter() {
			// Check for layer & aabb
			if l.overlap(l2) {
				if let Ok(s) = shapes.get(e2) {
					let shape2 = s.shape();
					let aabb2 = shape2.to_aabb(t2);

					if aabb.collides(&aabb2) {
						// ok check for actual collision now(this was the "broad phase")
						let p1 = shape.collide(&t, shape2, t2);
						let p2 = shape.collide(t2, shape, &t);

						// map the shortest pen correctly to p
						let p = match (p1, p2) {
							(Some(p1), Some(p2)) => {
								if p1.length_squared() < p2.length_squared() {
									Some(p1)
								}
								else {
									Some(-p2)
								}
							},
							(Some(p), None) => Some(p),
							(None, Some(p)) => Some(-p),
							(None, None) => { None }, // ignore
						};
						if let Some(p) = p {
							// We have a collision, make to to update the stuck value for this entity
							t.add_translation(p);
							stuck.push(p.normalize());
						}
					}
				}
			}
		}
		let g = CollisionGraph {
			stuck,
			collides: Vec::new(),
		};

		let _ = graph.insert(e, g);
	}

	// now loop over the kin_con and their events
	for c in ccoll.iter() {
		// make sure that indeed the entity is `With<Vel>`
		if let Ok(()) = kins_con.get(c.entity_a) {
			// either insert the normal to the stuck list or create a new list containing that normal 
			match graph.get_mut(&c.entity_a) {
				Some(s) => s.stuck.push(c.normal),
				None => {
					let g = CollisionGraph {
						stuck: Vec::from([c.normal]),
						collides: Vec::new(),
					};
					graph.insert(c.entity_a, g);
				},
			};
		}
	}
}

pub fn populate_graph_system(
	mut coll_order : ResMut<CollisionOrder>,
	mut graphs : ResMut<GraphMap>,
	shapes : Query<&CollisionShape>,
	// bodies
	kins : Query<(Entity, &Transform2D, &CollisionLayer),(Without<Vel>, Without<StaticBody>, Without<Sensor>)>,
	kins_con : Query<(Entity, &Transform2D, &CollisionLayer), With<Vel>>,
) {
	// we want to loop over kins(and kins_con)
	let graph = &mut graphs.0;
	let order = &mut coll_order.0;

	order.clear();

	for  (i, (e1, t1, l1)) in kins.iter().enumerate() {
		let aabb1 = match shapes.get(e1)  {
			Ok(s) => s.shape().to_aabb(t1),
			Err(_) => continue
		};

		let mut collides = Vec::<Entity>::new();

		// x Kinematic
		for (e2, t2, l2) in kins.iter().skip(i + 1) {
			if l1.overlap(l2) {
				let aabb2 = match shapes.get(e2) {
					Ok(s) => s.shape().to_aabb(t2),
					Err(_) => continue,
				};

				if aabb1.collides(&aabb2) {
					collides.push(e2);
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
					collides.push(e2);
				}
			}
		}

		order.push(e1);

		match graph.get_mut(&e1) {
			Some(g) => g.collides = collides,
			None => {
				let g = CollisionGraph {
					stuck : Vec::new(),
					collides,
				};
				graph.insert(e1, g);
			},
		}
	}
}

pub fn solve_graph_system(
	mut graphs : ResMut<GraphMap>,
	coll_order : Res<CollisionOrder>,
	shapes : Query<&CollisionShape>,
	mut transforms : Query<&mut Transform2D>,
	mut coll_writer : EventWriter<CollisionEvent>,
) {
	let graph = &mut graphs.0;

	for e in coll_order.0.iter() {
		// Get the collision graph
		if let Some(mut g) = graph.remove(e) {
			let non_solvable = solver(graph, &shapes, &mut transforms, &mut coll_writer, *e, &mut g, Vec2::ZERO);
		}
	}
	
}

/// Returns the movement `curr_e` could make
fn solver(
	graph : &mut HashMap<Entity, CollisionGraph>,
	shapes : &Query<&CollisionShape>,
	trans : &mut Query<&mut Transform2D>,
	writer : &mut EventWriter<CollisionEvent>,
	e : Entity,
	g : &mut CollisionGraph,
	mut ntm : Vec2, // ntm - needs to move
) -> Vec2 {
	const NTM_EPSILON : f32 = 0.0001;

	// extract the information about ourselves
	let t = match trans.get_component::<Transform2D>(e) {
		Ok(t) => t.clone(),
		Err(_) => return Vec2::ZERO,
	};
	let s = match shapes.get(e) {
		Ok(s) => s.shape(),
		Err(_) => return Vec2::ZERO,
	};
	
	// gather the penetration we need to solve
	let mut nodes = HashMap::<Entity, (CollisionGraph, Vec2)>::default();

	for &e2 in g.collides.iter() {
		let t2 = match trans.get_component::<Transform2D>(e2) {
			Ok(t) => t,
			Err(_) => return Vec2::ZERO,
		};
		let s2 = match shapes.get(e2) {
			Ok(s) => s.shape(),
			Err(_) => return Vec2::ZERO,
		};

		if let Some(p) = get_coll(s,&t,s2,t2) {
			if let Some(mut g2) = graph.remove(&e2) {
				let normal = -p.normalize();
				g2.stuck.push(normal);

				// each object should solve half the collision be default(as i just decided it should be)

				ntm += 0.5 * p;

				let not_solved = -solver(graph, shapes, trans, writer, e2, &mut g2, -0.5 * p);
				ntm += not_solved;

				if not_solved.length_squared() > NTM_EPSILON {
					g.stuck.push(not_solved.normalize());
				}
				else {
					nodes.insert(e2, (g2, normal));
				}
			}
		}
	}
	// calculate initial unsolvealbe
	let mut unsolveable = Vec2::ZERO;



	loop {

	}




	Vec2::ZERO
}

/// Penetration from `s1` point of view
fn get_coll(
	s1 : &dyn Shape,
	t1 : &Transform2D,
	s2 : &dyn Shape,
	t2 : &Transform2D,
) -> Option<Vec2> {
	let p1 = s1.collide(t1,s2,t2);
	let p2 = s2.collide(t2,s1,t1);

	match (p1,p2) {
		(Some(p1), Some(p2)) => {
			if p1.length_squared() < p2.length_squared() {
				Some(p1)
			}
			else {
				Some(-p2)
			}
		},
		(Some(p), None) => Some(p),
		(None, Some(p)) => Some(-p),
		_ => None
	}
}