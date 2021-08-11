use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) fn insert_physics_resources(app : &mut AppBuilder) {
    app.insert_resource(Gravity::default())
        .insert_resource(Friction::default())
        .insert_resource(AngFriction::default())
        .insert_resource(FloorAngle(0.7))
        .insert_resource(TransformMode::XY);
}

/// Gravity,
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Gravity(pub Vec2);
impl Default for Gravity {
    fn default() -> Self {
        Gravity(Vec2::new(0.0, -540.0))
    }
}

/// Global friction
#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Friction {
    /// Friction's "plane of action" normal
    ///
    /// Needs to be normalized(normal.len = 1)!!!
    pub normal : Vec2,
    /// How strong the force of friction is(default - 400.0)
    pub strength : f32,
}
impl Default for Friction {
    fn default() -> Self {
        Friction {
            normal : Vec2::Y,
            strength : 400.0,
        }
    }
}

/// Global angular friction
#[derive(Debug, Default, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct AngFriction(pub f32);

/// What angles are considered floor/wall/ceilling
///
/// a number between 0-1 representing 'normal.dot(-gravity)'
///
/// floor >= floor_angle // wall.abs() < floor_angle // ceil <= -floor_angle
///
/// Defaults to 0.7
pub struct FloorAngle(pub f32);

/// Which plane acts as the XY plane, rotation axis is the perpendicular axis
#[derive(Debug, Clone, Copy)]
pub enum TransformMode {
    XY,
    XZ,
    YZ,
}
impl TransformMode {
    /// Returns the position from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_position(
        &self,
        transform : &GlobalTransform,
    ) -> Vec2 {
        let t = transform.translation;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the rotation from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_rotation(
        &self,
        transform : &GlobalTransform,
    ) -> f32 {
        let t = transform.rotation;

        match self {
            TransformMode::XY => t.z,
            TransformMode::XZ => t.y,
            TransformMode::YZ => t.x,
        }
    }
    /// Returns the scale from a given `&GlobalTransform` and `TransformMode`
    pub fn get_global_scale(
        &self,
        transform : &GlobalTransform,
    ) -> Vec2 {
        let t = transform.scale;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the position from a given `&Transform` and `TransformMode`
    pub fn get_position(
        &self,
        transform : &Transform,
    ) -> Vec2 {
        let t = transform.translation;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Returns the rotation from a given `&Transform` and `TransformMode`
    pub fn get_rotation(
        &self,
        transform : &Transform,
    ) -> f32 {
        let t = transform.rotation;

        match self {
            TransformMode::XY => t.z,
            TransformMode::XZ => t.y,
            TransformMode::YZ => t.x,
        }
    }
    /// Returns the scale from a given `&Transform` and `TransformMode`
    pub fn get_scale(
        &self,
        transform : &Transform,
    ) -> Vec2 {
        let t = transform.scale;

        match self {
            TransformMode::XY => Vec2::new(t.x, t.y),
            TransformMode::XZ => Vec2::new(t.x, t.z),
            TransformMode::YZ => Vec2::new(t.y, t.z),
        }
    }
    /// Sets position based on `TransformMode`
    pub fn set_position(
        &self,
        transform : &mut Transform,
        pos : Vec2,
    ) {
        let t = transform.translation;

        transform.translation = match self {
            TransformMode::XY => Vec3::new(pos.x, pos.y, t.z),
            TransformMode::XZ => Vec3::new(pos.x, t.y, pos.y),
            TransformMode::YZ => Vec3::new(t.x, pos.x, pos.y),
        };
    }
    /// Sets rotation based on `TransformMode` (erase previus rotation)
    pub fn set_rotation(
        &self,
        transform : &mut Transform,
        rot : f32,
    ) {
        // TODO make it persist the other axis rotations, i dont understand quaternions
        transform.rotation = match self {
            TransformMode::XY => Quat::from_rotation_z(rot),
            TransformMode::XZ => Quat::from_rotation_y(rot),
            TransformMode::YZ => Quat::from_rotation_x(rot),
        }
    }
}
