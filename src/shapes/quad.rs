use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
};
use glam::DVec3;
use std::{ops::Range, sync::Arc};

pub struct Quad {
    q: DVec3,
    u: DVec3,
    v: DVec3,
    material: Arc<dyn Material + Send + Sync>,
    bbox: AABB,
    normal: DVec3,
    d: f64,
    w: DVec3,
}

impl Quad {
    pub fn new(q: DVec3, u: DVec3, v: DVec3, material: Arc<dyn Material + Send + Sync>) -> Self {
        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        let bbox = AABB::new(q, q + u + v).pad();

        Self {
            q,
            u,
            v,
            material,
            bbox,
            normal,
            d,
            w,
        }
    }

    /// Given the hit point in plane coordinates, return None if it is outside the
    /// primitive, otherwise return UV coordinates.
    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        if !(0. ..=1.).contains(&a) || !(0. ..=1.).contains(&b) {
            return None;
        }

        // a,b == u,v
        Some((a, b))
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.dir);

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        // Return none if the hit point parameter t is outside the ray interval.
        let t = (self.d - self.normal.dot(ray.orig)) / denom;
        if !interval.contains(&t) {
            return None;
        }

        // Determine the hit point lies within the planar shape using its plane coordinates.
        let intersection = ray.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(planar_hitpt_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hitpt_vector));

        Self::is_interior(alpha, beta).map(|(u, v)| {
            HitRecord::with_face_normal(
                intersection,
                self.normal,
                t,
                u,
                v,
                ray,
                self.material.as_ref(),
            )
        })
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
