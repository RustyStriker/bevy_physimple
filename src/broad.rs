use std::any::TypeId;
use bevy::{ecs::component::Component, prelude::*};
use crate::{bodies::*, prelude::PhysicsSettings, shapes::*};

#[derive(Clone, Copy, Debug)]
pub enum ShapeType {
	Square,
	Circle,
	None,
}
impl ShapeType {
	pub fn from_id(id : TypeId) -> ShapeType {
		if id == TypeId::of::<Square>() {
			ShapeType::Square
		}
		else if id == TypeId::of::<Circle>() {
			ShapeType::Circle
		}
		else {
			ShapeType::None
		}
	}
}
pub struct ObbData {
	pub(crate) entity : Entity,
	pub(crate) aabb : Aabb,
	pub(crate) shape_type : ShapeType,
	/// True - sensor, False - static
	pub(crate) sensor : bool,
}
pub struct ObbDataKinematic {
	pub(crate) entity : Entity,
	pub(crate) aabb : Aabb,
	pub(crate) shape_type : ShapeType,
}

/// Simply pushes ObbData and ObbDataKinematic into the event system for every shape
pub fn broad_phase_system<T>(
	settings : Res<PhysicsSettings>,
	statics : Query<(Entity, &GlobalTransform,&T), With<StaticBody2D>>, 
	sensors : Query<(Entity, &GlobalTransform,&T), With<Sensor2D>>,
	kinematics : Query<(Entity, &GlobalTransform,&T, &KinematicBody2D)>,
	mut writer : EventWriter<ObbData>,
	mut writer_kin : EventWriter<ObbDataKinematic>,
)
where
	T : Shape + Component,
{
	let shape_type = TypeId::of::<T>();
	let shape_type = ShapeType::from_id(shape_type);

	let tm = settings.transform_mode;

	// Static bodies
	for (e, t, s) in statics.iter() {
		let data = ObbData {
			entity: e,
			aabb: s.to_aabb(Transform2D::from((t, tm))),
			shape_type,
			sensor: false,
		};
		writer.send(data);
	}
	// Sensors :D
	for (e, t, s) in sensors.iter() {
		let data = ObbData {
			entity: e,
			aabb: s.to_aabb(Transform2D::from((t, tm))),
			shape_type,
			sensor: true,
		};
		writer.send(data);
	}
	// Kinematic stuff are complex af
	for (e, t, s, k) in kinematics.iter() {
		let t = Transform2D::from((t, tm));
		let t = Transform2D {
			translation : k.prev_position,
			..t
		};

		let data = ObbDataKinematic {
			entity: e,
			aabb: s.to_aabb(t),
			shape_type,
		};
		writer_kin.send(data);
	}

}