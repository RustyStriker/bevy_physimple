use bevy::prelude::*;

use crate::transform_mode::TransformMode;

/// This is a temporary struct until bevy gets it own `Transform2D` struct
#[derive(Clone, Debug, Reflect, Default)]
pub struct Transform2D {
    translation: Vec2,
    rotation: f32,
    scale: Vec2,
    translation_buffer: Vec2,
    rotation_buffer: f32,
}
impl Transform2D {
    pub fn new(translation : Vec2, rotation : f32, scale : Vec2) -> Transform2D {
        Transform2D {
            translation,
            rotation,
            scale,
            ..Default::default()
        }
    }

    // Getters
    pub fn translation(&self) -> Vec2 {
        self.translation
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn scale(&self) -> Vec2 {
        self.scale
    }
    // Adders
    pub fn add_translation(&mut self, amount : Vec2) {
        self.translation += amount;
        self.translation_buffer += amount;
    }
    pub fn add_rotation(&mut self, amount : f32) {
        self.rotation += amount;
        self.rotation += amount;
    }
    // Setters
    pub fn set_translation(&mut self, new : Vec2) {
        let original = self.translation - self.translation_buffer;
        self.translation = new;
        self.translation_buffer = new - original;
    }
    pub fn set_rotation(&mut self, new : f32) {
        let original = self.rotation - self.rotation_buffer;
        self.rotation = new;
        self.rotation_buffer = new - original;
    }
    pub fn apply_buffers(&self, transform : &mut Transform, trans_mode : TransformMode) {
        let (tb, rb) = (self.translation_buffer, self.rotation_buffer);

        let t = trans_mode.get_position(&transform);
        let r = trans_mode.get_rotation(&transform);

        trans_mode.set_position(transform, t + tb);
        trans_mode.set_rotation(transform, r + rb);
    }

    // systems

	/// Syncs from `GlobalTransform` to `Transform2D`
	pub fn sync_from_global_transform(
		trans_mode : Res<TransformMode>,
		mut query : Query<(&mut Transform2D, &GlobalTransform)>,
	) {
		for (mut t, gt) in query.iter_mut() {
			*t = (gt, *trans_mode).into();
		}
	}
	/// Syncs from `Transform2D` to `Transform`
	pub fn sync_to_transform(
		trans_mode : Res<TransformMode>,
		mut q : Query<(&Transform2D, &mut Transform)>,
	) {
		for (t2, mut mt) in q.iter_mut() {
			t2.apply_buffers(&mut mt, *trans_mode);
		}
	}
    /// Automatically inserts a Transform2D component for each new CollisionShape
    pub fn auto_insert_system(
        mut coms : Commands,
        q : Query<Entity, Added<crate::prelude::CollisionShape>>,
    ) {
        for e in q.iter() {
            coms.entity(e).insert(Transform2D::default());
        }
    }

}

impl From<(&GlobalTransform, TransformMode)> for Transform2D {
    fn from((trans, mode): (&GlobalTransform, TransformMode)) -> Self {
        let t = trans.translation;
        let q = trans.rotation;
        let s = trans.scale;

        // the weird conversion is from - it actually works...
        // https://en.wikipedia.org/wiki/Conversion_between_quaternions_and_Euler_angles#Quaternion_to_Euler_angles_conversion
        // they are correct, but it really looks made up...
        match mode {
            TransformMode::XY => Transform2D {
                translation: Vec2::new(t.x, t.y),
                rotation: (2.0 * (q.w * q.z + q.x * q.y))
                    .atan2(1.0 - 2.0 * (q.y * q.y + q.z * q.z)),
                scale: Vec2::new(s.x, s.y),
                ..Default::default()
            },
            TransformMode::XZ => Transform2D {
                translation: Vec2::new(t.x, t.z),
                rotation: {
                    let sinp = 2.0 * (q.w * q.y - q.z * q.x);
                    if sinp.abs() >= 1.0 {
                        0.5 * std::f32::consts::PI.copysign(sinp)
                    } else {
                        sinp.asin()
                    }
                },
                scale: Vec2::new(s.x, s.z),
                ..Default::default()
            },
            TransformMode::YZ => Transform2D {
                translation: Vec2::new(t.y, t.z),
                rotation: (2.0 * (q.w * q.x + q.y * q.z))
                    .atan2(1.0 - 2.0 * (q.x * q.x + q.y * q.y)),
                scale: Vec2::new(s.y, s.z),
                ..Default::default()
            },
        }
    }
}
impl From<(TransformMode, &GlobalTransform)> for Transform2D {
    fn from(v: (TransformMode, &GlobalTransform)) -> Self {
        (v.1, v.0).into()
    }
}
