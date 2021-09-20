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
        let q = transform.rotation;

        match self {
            TransformMode::XY => (2.0 * (q.w * q.z + q.x * q.y))
            .atan2(1.0 - 2.0 * (q.y * q.y + q.z * q.z)),
            TransformMode::XZ => {
                let sinp = 2.0 * (q.w * q.y - q.z * q.x);
                if sinp.abs() >= 1.0 {
                    0.5 * std::f32::consts::PI.copysign(sinp)
                } else {
                    sinp.asin()
                }
            },
            TransformMode::YZ => (2.0 * (q.w * q.x + q.y * q.z))
                .atan2(1.0 - 2.0 * (q.x * q.x + q.y * q.y)),
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
        let q = transform.rotation;

        match self {
            TransformMode::XY => (2.0 * (q.w * q.z + q.x * q.y))
            .atan2(1.0 - 2.0 * (q.y * q.y + q.z * q.z)),
            TransformMode::XZ => {
                let sinp = 2.0 * (q.w * q.y - q.z * q.x);
                if sinp.abs() >= 1.0 {
                    0.5 * std::f32::consts::PI.copysign(sinp)
                } else {
                    sinp.asin()
                }
            },
            TransformMode::YZ => (2.0 * (q.w * q.x + q.y * q.z))
                .atan2(1.0 - 2.0 * (q.x * q.x + q.y * q.y)),
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
        // This doesnt persist along other axes, but making it persist requires quite the overhead(and might not be useful at all)
        transform.rotation = match self {
            TransformMode::XY => Quat::from_rotation_z(rot),
            TransformMode::XZ => Quat::from_rotation_y(rot),
            TransformMode::YZ => Quat::from_rotation_x(rot),
        }
    }
}
