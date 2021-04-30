use bevy::prelude::*;

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
