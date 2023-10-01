use std::{ops::Range, sync::Arc};

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
};
use glam::DVec3;

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<dyn Material + Sync + Send>,
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Arc<dyn Material + Sync + Send>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let ac = ray.orig - self.center;
        let a = ray.dir.length_squared();
        let half_b = ray.dir.dot(ac);
        let c = ac.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if !interval.contains(&root) {
            root = (-half_b + sqrtd) / a;
            if !interval.contains(&root) {
                return None;
            }
        }

        let t = root;
        let point = ray.at(t);
        let outward_normal = (point - self.center) / self.radius;
        Some(HitRecord::with_face_normal(
            point,
            outward_normal,
            t,
            ray,
            self.material.as_ref(),
        ))
    }
}
