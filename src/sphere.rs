use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
};
use glam::DVec3;
use std::{
    f64::consts::PI,
    ops::{Neg, Range},
    sync::Arc,
};

#[derive(Clone)]
pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Arc<dyn Material + Sync + Send>,
    bbox: AABB,
}

impl Sphere {
    pub fn new(center: DVec3, radius: f64, material: Arc<dyn Material + Sync + Send>) -> Self {
        let rvec = DVec3::splat(radius);
        let bbox = AABB::new(center - rvec, center + rvec);
        Self {
            center,
            radius,
            material,
            bbox,
        }
    }

    fn get_sphere_uv(&self, point: &DVec3) -> (f64, f64) {
        // p: a given point on the sphere of radius one, centered at the origin.
        // u: returned value [0,1] of angle around the Y axis from X=-1.
        // v: returned value [0,1] of angle from Y=-1 to Y=+1.
        //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
        //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
        //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>

        let theta = point.y.neg().acos();
        let phi = point.z.neg().atan2(point.x) + PI;

        (phi / (2. * PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
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
        let (u, v) = self.get_sphere_uv(&outward_normal);
        Some(HitRecord::with_face_normal(
            point,
            outward_normal,
            t,
            u,
            v,
            ray,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
