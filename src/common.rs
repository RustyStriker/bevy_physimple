use bevy::prelude::*;

pub trait VecOp<T> {
    fn project(self, normal : T) -> T;
    fn slide(self, normal : T) -> T;
}

impl VecOp<Vec2> for Vec2 {
    fn project(
        self,
        n : Vec2,
    ) -> Vec2 {
        if n.is_normalized() {
            self.dot(n) * n
        }
        else {
            self // Just return the given a vector if n is not normalized
        }
    }
    fn slide(
        self,
        n : Vec2,
    ) -> Vec2 {
        if n.is_normalized() {
            self - self.project(n)
        }
        else {
            self
        }
    }
}
