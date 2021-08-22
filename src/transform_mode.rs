use bevy::prelude::*;

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
