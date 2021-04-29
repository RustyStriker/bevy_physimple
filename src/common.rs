//! Commmon type definitions for 2d and 3d physics simulation.
use bevy::prelude::*;

/// Extensions to the Bevy `Vec3` type
pub trait Vec3Ext {
    /// Returns the quaternion that describes the rotation from `self` to `other`.
    fn quat_between(&self, other: Vec3) -> Quat;
    /// Returns a projected copy of the current vector on other
    fn project(self, other : Vec3) -> Vec3;
    /// Returns a slided copy of the current vector on normal
    fn slide(self, normal : Vec3) -> Vec3;
}

impl Vec3Ext for Vec3 {
    fn quat_between(&self, other: Vec3) -> Quat {
        let dot = self.dot(other);
        if dot > 0.995 || dot < -0.995 {
            return Quat::IDENTITY;
        }

        let axis = self.cross(other);
        let angle = (self.length_squared() * other.length_squared()).sqrt() + dot;
        Quat::from_axis_angle(axis, angle)
    }
	fn project(self, n : Vec3) -> Vec3 {
		if n.is_normalized() {
			self.dot(n) * n
		}
		else {
			self // Just return the given a vector if n is not normalized
		}
	}
	fn slide(self, n : Vec3) -> Vec3 {
		if n.is_normalized() {
			self - self.project(n)
		}
		else {
			self
		}
	}
}
// Extentions to the Bevy 'Vec2' type
pub trait Vec2Ext {
    /// Returns a projected copy of the current vector on other
    fn project(self, other : Vec2) -> Vec2;
    /// Returns a slided copy of the current vector on normal
    fn slide(self, normal : Vec2) -> Vec2;
}
impl Vec2Ext for Vec2 {
	fn project(self, n : Vec2) -> Vec2 {
		if n.is_normalized() {
			self.dot(n) * n
		}
		else {
			self // Just return the given a vector if n is not normalized
		}
	}
	fn slide(self, n : Vec2) -> Vec2 {
		if n.is_normalized() {
			self - self.project(n)
		}
		else {
			self
		}
	}
}
