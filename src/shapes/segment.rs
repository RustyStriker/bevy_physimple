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

        let ap = np.dot(self.a);
        let bp = np.dot(self.b);

        let oap = np.dot(other.a);
        let obp = np.dot(other.b);

        let np_min = ap.min(bp);
        let np_max = ap.max(bp);

        let op_min = oap.min(obp);
        let op_max = oap.max(obp);

        if op_min <= np_max && op_max >= np_min {
            // im taking other to be flushed against self and not only solve the collision :(
            // we can define other by `other = A + t(B-A)`
            // or using n and np
            // `other = ((1-t)n.dot(A) + t*n.dot(B)) + ((1-t)np.dot(A) + t*np.dot(B))
            // and      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ n + ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ np
            // take the np part and solve for `t` (using x) and we will get
            // t = (x - np.dot(A)) / (np.dot(B) - np.dot(A))
            // so we need to find the corresponding position
            let x_min = np_min.max(op_min);
            let x_max = np_max.min(op_max);

            let t_min = (x_min - np.dot(other.a)) / (np.dot(other.b) - np.dot(other.a));
            let t_max = (x_max - np.dot(other.a)) / (np.dot(other.b) - np.dot(other.a));

            let y_min = (1.0 - t_min) * self.n.dot(other.a) + t_min * self.n.dot(other.b);
            let y_max = (1.0 - t_max) * self.n.dot(other.a) + t_max * self.n.dot(other.b);

            let min = y_min.min(y_max) - self.n.dot(self.a);
            // NOTE: self.n.dot(self.a) == self.n.dot(self.b) - assuming the normal is correct

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

        let ap = np.dot(self.a);
        let bp = np.dot(self.b);

        let pp = np.dot(point);

        let np_part = if pp >= ap.min(bp) && pp <= ap.max(bp) {
            0.0
        }
        else {
            let a = pp - ap;
            let b = pp - bp;
            if a.abs() < b.abs() {
                a
            }
            else {
                b
            }
        };

        (self.n.dot(point - self.a), np_part)
    }
}

#[cfg(test)]
mod segment_tests {
    use bevy::math::vec2;
    use super::*;

    const EPSILON : f32 = 0.0001; // f32::EPSILON is a tad too accurate for these tests

    #[test]
    fn no_collision() {
        let seg_a = Segment {
            a: vec2(10.0,0.0),
            b: vec2(-10.0,0.0),
            n: vec2(0.0,1.0),
        };

        let seg_b = Segment {
            a: vec2(10.0,5.0),
            b: vec2(-10.0,5.0),
            n: vec2(0.0,-1.0),
        };

        let a_b = seg_a.collide(seg_b);
        let b_a = seg_b.collide(seg_a);

        // Make sure both are equal
        assert_eq!(
            a_b,
            b_a
        );
        let amount = a_b.unwrap();
        assert!((amount - 5.0).abs() < EPSILON);
    }

    #[test]
    fn collision_test_1() {
        let a = Segment {
            a: vec2(10.0,0.0),
            b: vec2(-10.0,0.0),
            n: vec2(0.0,1.0),
        };

        let b = Segment {
            a: vec2(3.0,-1.0),
            b: vec2(-3.0,1.0),
            n: vec2(-3.0,-1.0).normalize(),
        };

        let a_b = a.collide(b);
        let b_a = b.collide(a);

        // Make sure both of them recognize the collisions
        assert_eq!(a_b.is_some(), b_a.is_some());

        assert!((a_b.unwrap() + 1.0).abs() < EPSILON); // a_b ~ -1.0
        // the b_a value is ~5.69 (calculated from the function) so i dont see a point in putting it here
    }
    #[test]
    fn collision_test_2() {
        let a = Segment {
            a: vec2(3.0,0.0),
            b: vec2(-1.0,0.0),
            n: vec2(0.0,1.0),
        };

        let b = Segment {
            a: vec2(6.0,-2.0),
            b: vec2(-3.0,1.0),
            n: vec2(-3.0,-1.0).normalize(),
        };

        let a_b = a.collide(b);
        let b_a = b.collide(b);

        // Make sure they both see collision
        assert_eq!(a_b.is_some(), b_a.is_some());

        assert!((a_b.unwrap() + 1.0).abs() < EPSILON); // a_b ~ -1.0 and f32::EPSILON is too accurate for this test
    }

    #[test]
    fn point_collisions() {
        let s = Segment {
            a: vec2(1.0,0.0),
            b: vec2(-1.0,0.0),
            n: vec2(0.0,1.0),
        };

        let ps = [
            (vec2(0.0,1.0), (1.0, 0.0)), 
            (vec2(1.0,1.0), (1.0,0.0)),
            (vec2(2.0,1.0), (1.0,-1.0)),
            (vec2(0.0,-1.0), (-1.0, 0.0)), 
        ];

        for (p, e) in ps {
            let r = s.collide_point(p);

            println!("r {:?} e {:?}",r,e);

            // Compare first result(on N)
            assert!((e.0 - r.0).abs() < EPSILON);
            // Compare second result(on NP)
            assert!((e.1 - r.1).abs() < EPSILON);
        }
    }
    
}