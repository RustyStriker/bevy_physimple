use bevy::prelude::*;

pub trait VecOp<T> {
    /// Projects the vector on the given normal
    fn project(
        self,
        normal : T,
    ) -> T;
    /// Slides the vector on the given normal
    fn slide(
        self,
        normal : T,
    ) -> T;
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
