use crate::physics_components::Transform2D;
use bevy::{math::Mat2, prelude::*};

mod aabb;
mod circle;
mod square;
mod capsule;
mod triangle;

pub use aabb::*;
pub use circle::*;
pub use square::*;
pub use capsule::*;
pub use triangle::*;

pub trait SAT {
    /// Gets the Axis Aligned Bounding Box of the shape
    fn aabb(&self, trans: &Transform2D) -> Aabb {
        let (xmin, xmax) = self.project(trans, Vec2::X);
        let (ymin, ymax) = self.project(trans, Vec2::Y);

        let min = Vec2::new(xmin, ymin);
        let max = Vec2::new(xmax, ymax);

        let extents = (max - min) * 0.5;
        let position = min + extents;

        Aabb { extents, position }
    }

    /// Gets the normals to use in the SAT algorithm(should simply be the normals of the edges)
    ///
    /// HINT: there is no need to give 2 parallel normals(as they produce the same results) 
    fn get_normals(&self, trans: &Transform2D) -> Box<dyn Iterator<Item = Vec2> + '_>;

    /// Gets the projection of the shape on the given normal
    ///
    /// (min, max)
    fn project(&self, trans: &Transform2D, normal: Vec2) -> (f32,f32);

    /// Gets the closest vertex to the given point, used for SAT vs Special shapes(Circle and Capsule)
    fn get_closest_vertex(&self, trans: &Transform2D, vertex: Vec2) -> Vec2;

    /// Gets the collision with a ray
    ///
    /// ray_origin: The tail of the ray
    ///
    /// ray_cast: The point(relative to ray_origin) the ray points to 
    fn ray(&self, trans: &Transform2D, ray_origin: Vec2, ray_cast:  Vec2) -> Option<f32>;
}

/// Collides 2 shapes and returns the MTV relative to a
///
/// MTV - Minimal Tranlsation Vector
pub fn collide(a: &CollisionShape, trans_a: &Transform2D, b: &CollisionShape, trans_b: &Transform2D) -> Option<Vec2> {
    if let CollisionShape::Multiple(v) = a {
        // If a is multiple shapes just break it up and attempt to combine the output
        let mut sum = Vec2::ZERO;
        for s in v {
            if let Some(c) = collide(s, trans_a, b, trans_b) {
                // I know we want to better check if we arnt already exiting the shape
                // but it seems like way to much extra complexity for now
                sum += c; 
            }
        }
        if sum.length_squared() < 0.01 {
            return None;
        }
        else {
            return Some(sum);
        }
        
    }
    // It looks weird i know, but we need to check for both a and b, if both are multiple we need to check on all T_T
    if let CollisionShape::Multiple(v) = b {
        // If a is multiple shapes just break it up and attempt to combine the output
        let mut sum = Vec2::ZERO;
        for s in v {
            if let Some(c) = collide(a, trans_a, s, trans_b) {
                // I know we want to better check if we arnt already exiting the shape
                // but it seems like way to much extra complexity for now
                sum += c; 
            }
        }
        if sum.length_squared() < 0.01 {
            return None;
        }
        else {
            return Some(sum);
        }
    }

    let sat_a = a.sat();
    let sat_b = b.sat();

    match (sat_a, sat_b) {
        (Some(a), Some(b)) => sat_normal(a, trans_a, b, trans_b),
        (Some(a), None) => sat_special(a, trans_a, b, trans_b), // Special vs sat
        (None, Some(b)) => sat_special(b, trans_b, a, trans_a).map(|c| -c), // Special vs sat - we need to flip here
        (None, None) => collide_special(a, trans_a, b, trans_b), // Special vs Special
    }
}

fn sat_normal(a: &dyn SAT, ta: &Transform2D, b: &dyn SAT, tb: &Transform2D) -> Option<Vec2> {
    let na = a.get_normals(ta);
    let nb = b.get_normals(tb);

    let mut minimal_dis = f32::INFINITY;
    let mut minimal_n = Vec2::ZERO;

    for n in na.chain(nb) {
        let (mina, maxa) = a.project(ta, n);
        let (minb, maxb) = b.project(tb, n);

        if mina < maxb && minb < maxa {
            // collision on this axis - lets get the mtv
            let p1 = maxb - mina;
            let p2 = minb - maxa;

            let p = if p1.abs() < p2.abs() { p1 } else { p2 };

            if p.abs() < minimal_dis.abs() {
                minimal_dis = p;
                minimal_n = n;
            }
        }
        else {
            // if we find a non colliding axis, we know they dont collide :D
            return None;
        }
    }
    Some(minimal_dis * minimal_n)
}

fn sat_special(a: &dyn SAT, ta: &Transform2D, b: &CollisionShape, tb: &Transform2D) -> Option<Vec2> {
    let na = a.get_normals(ta);
    let b_rot = Mat2::from_angle(tb.rotation());
    let nb = match b {
        CollisionShape::Circle(c) => {
            let offset = b_rot * c.offset;
            let v = a.get_closest_vertex(ta, tb.translation() + offset);
            (tb.translation() + offset - v).normalize()
        },
        CollisionShape::Capsule(c) => {
            let offset = b_rot * c.offset;
            let v = a.get_closest_vertex(ta, tb.translation() + offset);
            c.sat_normal(tb, v)
        }
        _ => panic!("Shouldn't happen, if this occur to you please report it as a bug(and how you got here)")
    };

    let mut minimal_dis = f32::INFINITY;
    let mut minimal_n = Vec2::ZERO;

    for n in na.chain([nb]) {
        let (mina, maxa) = a.project(ta, n);
        let (minb, maxb) = match b {
            CollisionShape::Circle(c) => {
                let center = tb.translation() + b_rot * c.offset;
                let center = center.dot(n);

                (center - c.radius, center + c.radius)
            },
            CollisionShape::Capsule(c) => c.project(tb, n),
            _ => panic!("If you paniced here, something is REALLY wrong")
        };

        if mina < maxb && minb < maxa {
            // collision on this axis - lets get the mtv
            let p1 = maxb - mina;
            let p2 = minb - maxa;

            let p = if p1.abs() < p2.abs() { p1 } else { p2 };

            if p.abs() < minimal_dis.abs() {
                minimal_dis = p;
                minimal_n = n;
            }
        }
        else {
            // if we find a non colliding axis, we know they dont collide :D
            return None;
        }
    }
    Some(minimal_dis * minimal_n)
}

fn collide_special(a: &CollisionShape, ta: &Transform2D, b: &CollisionShape, tb: &Transform2D) -> Option<Vec2> {
    #[allow(clippy::enum_glob_use)]
    use CollisionShape::*;
    
    match (a, b) {
        (Circle(a), Circle(b)) => {
            let ac = ta.translation() + Mat2::from_angle(ta.rotation()) * a.offset;
            let bc = tb.translation() + Mat2::from_angle(tb.rotation()) * b.offset;
            let d = ac - bc;
            let d_len = d.length();

            if d_len < a.radius + b.radius {
                // collision
                Some((a.radius + b.radius - d_len) * (d / d_len))
            }
            else {
                None
            }
        },
        (Circle(a), Capsule(b)) => collide_circle_capsule(a, ta, b, tb),
        (Capsule(a), Circle(b)) => collide_circle_capsule(b, tb, a, ta).map(|v| -v),
        (Capsule(a), Capsule(b)) => {
            let a_rot = Mat2::from_angle(ta.rotation());
            let b_rot = Mat2::from_angle(tb.rotation());

            // When you make 2 capsules obey SAT rules :D(they are still not fully SAT tho)

            let n1 = a_rot * Vec2::X;
            let n2 = b_rot * Vec2::X;

            // get the closer vertex of b(relative to a)
            let n3 = {
                let b1 = b_rot * Vec2::new(0.0,  b.half_height) + tb.translation() + b_rot * b.offset;
                let b2 = b_rot * Vec2::new(0.0, -b.half_height) + tb.translation() + b_rot * b.offset;

                let v = ta.translation() + a_rot * a.offset;

                let d1 = b1 - v;
                let d2 = b2 - v;

                if d1.length_squared() < d2.length_squared() {
                    d1.normalize_or_zero()
                }
                else {
                    d2.normalize_or_zero()
                }
            };

            let mut minimal_dis = f32::INFINITY;
            let mut minimal_n = Vec2::ZERO;

            for n in [n1,n2,n3] {
                let (mina, maxa) = a.project(ta, n);
                let (minb, maxb) = b.project(tb, n);

                if mina < maxb && minb < maxa {
                    // collision on this axis - lets get the mtv
                    let p1 = maxb - mina;
                    let p2 = minb - maxa;

                    let p = if p1.abs() < p2.abs() { p1 } else { p2 };

                    if p.abs() < minimal_dis.abs() {
                        minimal_dis = p;
                        minimal_n = n;
                    }
                }
                else {
                    // if we find a non colliding axis, we know they dont collide :D
                    return None;
                }
            }
            Some(minimal_dis * minimal_n)
        },
        _ => panic!("Something is missing, please report it on github(with the shapes used)"),
    }
}

fn collide_circle_capsule(a: &Circle, ta: &Transform2D, b: &Capsule, tb: &Transform2D) -> Option<Vec2> {
    let brot = Mat2::from_angle(tb.rotation());
    
    // get the distance of the circle's center to the capsule's center line
    let (ba, bb) = b.center_line(tb);

    let acenter = ta.translation() + Mat2::from_angle(ta.rotation()) * a.offset;

    let n = brot * Vec2::X;
    let p = brot * Vec2::Y;

    let bn = n.dot(ba); // n.dot(ba) should be equal n.dot(bb) should be equal n.dot(capsule_center)
    let bap = p.dot(ba);
    let bbp = p.dot(bb);
    
    let an = n.dot(acenter);
    let ap = p.dot(acenter);
    
    let bpmin = bap.min(bbp);
    let bpmax = bap.max(bbp);

    let dp = if ap > bpmax { ap - bpmax } else if ap < bpmin { ap - bpmin } else { 0.0 };

    let dis = n * (an - bn) + p * dp;

    let dis_n = dis.normalize();
    let dis_l = dis.dot(dis_n);

    if dis_l < (a.radius + b.radius) {
        Some(dis_n * (a.radius + b.radius - dis_l))
    } else {
        None
    }
}

/**
    # CollisionShape

    Enum which can hold all possible collision shapes.

    If you want to use a custom shape,
    you can do so by implementing the `SAT` trait for your shape(check the `convex` example),
    and box it.

    Alternatively, you can build it from a vector of `CollisionShape`,
    using `CollisionShape::Multiple`(see `showcase` example)
    
    Do note that this library is using the Seperate Axis Theorem, which doesnt work for concave shapes.
    (unless of course borken down into multiple convex shapes using `CollisionShape::Multiple`)
*/
#[derive(Component)]
pub enum CollisionShape {
    Square(Square),
    Triangle(Triangle),
    Circle(Circle),
    Capsule(Capsule),
    Multiple(Vec<CollisionShape>),
    Convex(Box<dyn SAT + Send + Sync>),
}
impl CollisionShape {
    pub fn sat(&self) -> Option<&dyn SAT> {
        match self {
            CollisionShape::Square(s) => Some(s),
            CollisionShape::Triangle(t) => Some(t),
            CollisionShape::Circle(_) => None,
            CollisionShape::Capsule(_) => None,
            CollisionShape::Multiple(_) => None,
            CollisionShape::Convex(s) => Some(s.as_ref())
        }
    }

    pub fn aabb(&self, t: &Transform2D) -> Aabb {
        if let Some(sat) = self.sat() {
            sat.aabb(t)
        }
        else {
            match self {
                CollisionShape::Circle(c) => c.aabb(t),
                CollisionShape::Capsule(c) => c.aabb(t),
                CollisionShape::Multiple(v) => {
                    // Make sure we have at least 1 shape :D
                    assert!(v.len() != 0, "CollisionShape::Multiple cannot be empty!");

                    let (mut min, mut max) = v[0].aabb(t).min_max();

                    // Skip the first as we already checked him
                    for s in v.iter().skip(1) {
                        let (sn, sx) = s.aabb(t).min_max();
                        min = min.min(sn);
                        max = max.max(sx);
                    }
                    Aabb::from_min_max(min, max)
                }
                _ => panic!("Something is missing, please report on github(with the shape used)"),
            }
        }
    }

    pub fn ray(&self, trans: &Transform2D, ray_origin: Vec2, ray_cast: Vec2) -> Option<f32> {
        if let Some(sat) = self.sat() {
            sat.ray(trans, ray_origin, ray_cast)
        }
        else {
            match self {
                CollisionShape::Circle(c) => c.ray(trans, ray_origin, ray_cast),
                CollisionShape::Capsule(c) => c.ray(trans, ray_origin, ray_cast),
                CollisionShape::Multiple(v) => {
                    // Make sure we have at least 1 shape :D
                    assert!(v.len() != 0, "CollisionShape::Multiple cannot be empty!");
                    
                    let mut res = None;
                    for s in v {
                        if let Some(r) = s.ray(trans, ray_origin, ray_cast) {
                            if r < res.unwrap_or(f32::INFINITY) {
                                res = Some(r);
                            }
                        }
                    }
                    res
                }
                _ => panic!("Something is missing, please report on github(with the shape used)"),
            }
        }
    }
}
impl Default for CollisionShape {
    fn default() -> Self {
        CollisionShape::Square(Square::default())
    }
}

#[cfg(test)]
mod sat_tests {
    use super::*;

    use std::f32::consts::PI;
    // Use a much higher value of epsilon due to the trigo functions in the rotation calculations having
    //  around 0.0000005 miss
    const EPSILON: f32 = 0.001;

    #[test]
    fn squares() {
        let s1 = Square {
            offset: Vec2::ZERO,
            extents: Vec2::splat(1.0),
        };

        let t1 = Transform2D::new(
            Vec2::ZERO,
            0.0,
            Vec2::splat(1.0),
        );

        let s2 = Square {
            offset: Vec2::ZERO,
            extents: Vec2::splat(1.0),
        };

        let t2 = Transform2D::new(
            Vec2::new(1.5, 0.0),
            0.0,
            Vec2::splat(1.0),
        );

        let cs1 = CollisionShape::Square(s1);
        let cs2 = CollisionShape::Square(s2);

        assert_eq!(
            collide(&cs1, &t1, &cs2, &t2),
            Some(Vec2::new(-0.5, 0.0))
        );
        assert_eq!(
            collide(&cs2, &t2, &cs1, &t1),
            Some(Vec2::new(0.5,0.0))
        );
    }
    #[test]
    fn squares_rotation() {
        let a = Square {
            offset: Vec2::ZERO,
            extents: Vec2::splat(1.0),
        };
        let ta = Transform2D::new(
            Vec2::ZERO,
            0.0,
            Vec2::splat(1.0)
        );

        let b = Square {
            offset: Vec2::ZERO,
            extents: Vec2::splat(1.0),
        };
        let tb = Transform2D::new(
            Vec2::new(2.0, 0.5),
            PI * 0.25,
            Vec2::splat(1.0),
        );

        let c = collide(&CollisionShape::Square(a), &ta, &CollisionShape::Square(b), &tb);

        assert!((c.unwrap() + Vec2::new(2.0_f32.sqrt() - 1.0, 0.0)).length() < EPSILON);
    }
}
