use crate::{bodies::*, prelude::PhysicsSettings, shapes::*};
use bevy::{ecs::component::Component, prelude::*};
use std::any::TypeId;

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
    pub(crate) coll_layer : u8,
    pub(crate) coll_mask : u8,
}
pub struct ObbDataKinematic {
    pub(crate) entity : Entity,
    pub(crate) aabb : Aabb,
    pub(crate) shape_type : ShapeType,
}

/// Simply pushes ObbData and ObbDataKinematic into the event system for every shape
pub fn broad_phase_system<T>(
    settings : Res<PhysicsSettings>,
    statics : Query<(Entity, &GlobalTransform, &T, &StaticBody2D)>,
    sensors : Query<(Entity, &GlobalTransform, &T, &Sensor2D)>,
    kinematics : Query<(Entity, &GlobalTransform, &T, &KinematicBody2D)>,
    mut writer : EventWriter<ObbData>,
    mut writer_kin : EventWriter<ObbDataKinematic>,
) where
    T : Shape + Component,
{
    let shape_type = TypeId::of::<T>();
    let shape_type = ShapeType::from_id(shape_type);

    let tm = settings.transform_mode;

    // Static bodies
    for (e, t, s, sb) in statics.iter() {
        if sb.active {
            let data = ObbData {
                entity : e,
                aabb : s.to_aabb(Transform2D::from((t, tm))),
                shape_type,
                sensor : false,
                coll_layer : sb.layer,
                coll_mask : sb.mask,
            };
            writer.send(data);
        }
    }
    // Sensors :D
    for (e, t, s, sen) in sensors.iter() {
        let data = ObbData {
            entity : e,
            aabb : s.to_aabb(Transform2D::from((t, tm))),
            shape_type,
            sensor : true,
            coll_layer : sen.layer,
            coll_mask : sen.mask,
        };
        writer.send(data);
    }
    // Kinematic stuff are complex af
    for (e, t, s, k) in kinematics.iter() {
        if k.active {
            let t = Transform2D::from((t, tm));

            let data = ObbDataKinematic {
                entity : e,
                aabb : s.to_aabb(t),
                shape_type,
            };
            writer_kin.send(data);
        }
    }
}
