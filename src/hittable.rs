use crate::ray::Ray;
use glam::DVec3;
use std::ops::Range;

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord>;
}

pub struct HitRecord {
    pub point: DVec3,
    pub normal: DVec3,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    /// NOTE: the parameter `outward_normal` is assumed to have unit length.
    pub fn with_face_normal(point: DVec3, outward_normal: DVec3, t: f64, ray: &Ray) -> Self {
        let (front_face, normal) = Self::calc_face_normal(ray, outward_normal);

        Self {
            point,
            normal,
            t,
            front_face,
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

    // TODO: is this necessary?
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: DVec3) {
        let (front_face, normal) = Self::calc_face_normal(ray, outward_normal);

        self.front_face = front_face;
        self.normal = normal;
    }
}

pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let (_closest, hit) = self.objects.iter().fold((interval.end, None), |acc, obj| {
            if let Some(hit) = obj.hit(ray, interval.start..acc.0) {
                (hit.t, Some(hit))
            } else {
                acc
            }
        });

        hit
    }
}
