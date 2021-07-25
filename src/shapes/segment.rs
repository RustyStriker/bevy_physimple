use bevy::prelude::{Reflect, Vec2};

/// Simple struct to represent a segment from a to b
#[derive(Clone, Copy, Reflect, Debug)]
pub struct Segment {
    /// Point a
    pub a : Vec2,
    /// Point b
    pub b : Vec2,
    /// Normal
    pub n : Vec2,
}
impl Segment {
    /// Returns the `a` where `penetration = a * self.normal`
    ///
    /// if `a > 0.0` -> no penetration happend, this is the distance
    pub fn collide(
        self,
        other : Segment,
    ) -> Option<f32> {
        let np = self.n.perp();
        let c = (self.a + self.b) * 0.5;

        let ap = np.dot(self.a - c);
        let bp = np.dot(self.b - c);

        let oap = np.dot(other.a - c);
        let obp = np.dot(other.b - c);

        let np_min = ap.min(bp);
        let np_max = ap.max(bp);

        let op_min = oap.min(obp);
        let op_max = oap.max(obp);

        if op_min <= np_max && op_max >= np_min {
            let oan = self.n.dot(other.a - c);
            let obn = self.n.dot(other.b - c);

            let min = oan.min(obn);

            Some(min)
        }
        else {
            None
        }
    }

    /// Returns the minimum distance between the segment and a given point
    ///
    /// Returns: (length on normal, length perp to normal)
    pub fn collide_point(
        self,
        point : Vec2,
    ) -> (f32, f32) {
        let np = self.n.perp();
        let c = (self.a + self.b) * 0.5;

        let ap = np.dot(self.a - c);
        let bp = np.dot(self.b - c);

        let pp = np.dot(point - c);

        let np_part = if pp >= ap.min(bp) && pp <= ap.max(bp) {
            0.0
        }
        else {
            let a = pp - ap;
            let b = pp - bp;
            if a.abs() > b.abs() {
                a
            }
            else {
                b
            }
        };

        (self.n.dot(point - c), np_part)
    }
}
