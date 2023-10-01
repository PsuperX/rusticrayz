use crate::{material::Material, ray::Ray};
use glam::DVec3;
use std::ops::Range;

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord>;
}

pub struct HitRecord<'a> {
    pub point: DVec3,
    pub normal: DVec3,
    pub t: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    /// NOTE: the parameter `outward_normal` is assumed to have unit length.
    pub fn with_face_normal(
        point: DVec3,
        outward_normal: DVec3,
        t: f64,
        ray: &Ray,
        material: &'a dyn Material,
    ) -> Self {
        let (front_face, normal) = Self::calc_face_normal(ray, outward_normal);

        Self {
            point,
            normal,
            t,
            front_face,
            material,
        }
    }

    fn calc_face_normal(ray: &Ray, outward_normal: DVec3) -> (bool, DVec3) {
        let front_face = ray.dir.dot(outward_normal) < 0.;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        (front_face, normal)
    }
}

impl<T: Hittable> Hittable for Vec<T> {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let (_closest, hit) = self.iter().fold((interval.end, None), |acc, obj| {
            if let Some(hit) = obj.hit(ray, interval.start..acc.0) {
                (hit.t, Some(hit))
            } else {
                acc
            }
        });

        hit
    }
}
