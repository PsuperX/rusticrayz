use crate::{aabb::AABB, material::Material, ray::Ray};
use glam::DVec3;
use std::ops::Range;

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord>;

    fn bounding_box(&self) -> &AABB;
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

#[derive(Default, Clone)]
pub struct HittableList<T: Hittable> {
    pub objects: Vec<T>,
    bbox: AABB,
}

impl<T: Hittable> HittableList<T> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: AABB::empty(),
        }
    }

    pub fn from_vec(objects: Vec<T>) -> Self {
        Self {
            bbox: objects
                .iter()
                .fold(AABB::default(), |acc, cur| acc.merge(cur.bounding_box())),
            objects,
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: T) {
        self.bbox = self.bbox.merge(object.bounding_box());
        self.objects.push(object);
    }
}

impl<T: Hittable> Hittable for HittableList<T> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        let (_closest, hit) = self.objects.iter().fold((interval.end, None), |acc, obj| {
            if let Some(hit) = obj.hit(ray, &(interval.start..acc.0)) {
                (hit.t, Some(hit))
            } else {
                acc
            }
        });

        hit
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}

impl<T: Hittable> From<Vec<T>> for HittableList<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            bbox: value.iter().fold(AABB::default(), |acc, cur| {
                AABB::merge(&acc, cur.bounding_box())
            }),
            objects: value,
        }
    }
}

impl Hittable for Box<dyn Hittable> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        self.as_ref().hit(ray, interval)
    }

    fn bounding_box(&self) -> &AABB {
        self.as_ref().bounding_box()
    }
}

impl Hittable for Box<dyn Hittable + Send> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        self.as_ref().hit(ray, interval)
    }

    fn bounding_box(&self) -> &AABB {
        self.as_ref().bounding_box()
    }
}

impl Hittable for Box<dyn Hittable + Sync> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        self.as_ref().hit(ray, interval)
    }

    fn bounding_box(&self) -> &AABB {
        self.as_ref().bounding_box()
    }
}

impl Hittable for Box<dyn Hittable + Sync + Send> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        self.as_ref().hit(ray, interval)
    }

    fn bounding_box(&self) -> &AABB {
        self.as_ref().bounding_box()
    }
}
