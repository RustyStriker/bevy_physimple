use bevy::math::Mat2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Transform2D, SAT};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    /// Verts of the triangle...
    verts: [Vec2; 3],
    /// The normals... remember to update them every time you update the verts
    /// 
    /// `normals[i] = normal between verts[i], verts[i + 1]`
    normals: [Vec2; 3],
}
impl Triangle {
    /// Creates a triangle from 3 verts
    pub fn new(v1: Vec2, v2: Vec2, v3: Vec2) -> Triangle {
        let mut t = Triangle { 
            verts: [v1, v2, v3], 
            normals: [
                (v2 - v1).perp().normalize(), 
                (v3 - v2).perp().normalize(), 
                (v1 - v2).perp().normalize()
            ],
        };
        t.validate_normals();
        t
    }
    /// Validates and flips(where needed) normals
    pub fn validate_normals(&mut self) {
        let v = &self.verts;
        let center = (v[0] + v[1] + v[2]) / 3.0;
        let c = [
            (v[1] - v[0]) * 0.5 + v[0] - center, 
            (v[2] - v[1]) * 0.5 + v[1] - center, 
            (v[0] - v[2]) * 0.5 + v[2] - center,
        ];
        let n = &mut self.normals;

        for i in 0..3 {
            if n[i].dot(c[i]) < 0.0 {
                n[i] = -n[i];
            }
        }
    }
    /// Updates a single vert(and the 2 needed normals as well :D)
    /// 
    /// WARNING: Does not validate normals
    /// 
    /// Also `i` needs to satisfy `i <= 2 && i >= 0` (the second is satisfied due to `i` being a `usize` tho)
    pub fn update_vert(&mut self, i: usize, nv: Vec2) {
        if i > 2 {
            panic!("Cannot update_vert with i = {}(as it is over 2", i);
        }

        self.verts[i] = nv;
        self.normals[i] = (self.verts[i + 1 % 3] - self.verts[i]).perp().normalize();
        self.normals[i - 1 % 3] = (self.verts[i] - self.verts[i - 1 % 3]).perp().normalize();
    }
    /// Updates the first vertex
    pub fn update_v1(&mut self, nv: Vec2) {
        self.update_vert(0, nv);
        self.validate_normals();
    }
    /// Updates the second vertex
    pub fn update_v2(&mut self, nv: Vec2) {
        self.update_vert(1, nv);
        self.validate_normals();
    }
    /// Updates the third vertex
    pub fn update_v3(&mut self, nv: Vec2) {
        self.update_vert(2, nv);
        self.validate_normals();
    }
}
impl SAT for Triangle {
    fn get_normals(&self, trans: &Transform2D) -> Box<dyn Iterator<Item = bevy::prelude::Vec2> + '_> {
        let rot = Mat2::from_angle(trans.rotation());

        Box::new(self.normals.iter().map(move |n| rot * *n))
    }

    fn project(&self, trans: &Transform2D, normal: Vec2) -> (f32,f32) {
        let rot = Mat2::from_angle(trans.rotation());

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for v in self.verts {
            let v = rot * v + trans.translation();
            let proj = v.dot(normal);

            min = min.min(proj);
            max = max.max(proj);
        }

        (min, max)
    }

    fn get_closest_vertex(&self, trans: &Transform2D, vertex: Vec2) -> Vec2 {
        let rot = Mat2::from_angle(trans.rotation());

        let mut cv = Vec2::ZERO;
        let mut cls = f32::INFINITY;

        for v in self.verts {
            let v = rot * v + trans.translation();
            let ls = (v - vertex).length_squared();

            if ls < cls {
                cls = ls;
                cv = v;
            }
        } 

        cv
    }

    fn ray(&self, trans: &Transform2D, ray_origin: Vec2, ray_cast: Vec2) -> Option<f32> {
        let n = ray_cast.normalize();
        let p = n.perp();
        let r_len = ray_cast.dot(n);

        let rot = Mat2::from_angle(trans.rotation());
        let mut coll = None;

        for i in 0..3 {
            let es = rot * self.verts[i] + trans.translation();
            let ee = rot * self.verts[i + 1 % 3] + trans.translation();
            
            let es_p = es.dot(p);
            let ee_p = ee.dot(p);

            let ep_min = es_p.min(ee_p);
            let ep_max = es_p.max(ee_p);

            let rp = ray_origin.dot(p);

            if ep_min < rp && ep_max > rp {
                // Got a collision on the "Y" axis
                let en_min = ee.dot(n).min(es.dot(n));
                let en_max = ee.dot(n).max(es.dot(n));

                // No need to do min/max here because r_org will always be less than r_end
                let r_min = ray_origin.dot(n);
                let r_max = (ray_origin + ray_cast).dot(n);

                if (en_min > r_min && en_min < r_max) || (en_max > r_min && en_max < r_max) {
                    // Needs to find the intersection here
                    let t = (rp - es_p) / (ee_p - es_p);

                    let y = (1.0 - t) * n.dot(es) + t * n.dot(ee);
                    let y = y - n.dot(ray_origin);

                    if y <= r_len && y >= 0.0 && y < coll.unwrap_or(f32::INFINITY) {
                        coll = Some(y)
                    }
                }
            }
        }
        coll
    }
}