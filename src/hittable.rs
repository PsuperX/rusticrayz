use crate::{aabb::AABB, material::Material, ray::Ray};
use glam::{dvec3, DVec3};
use itertools::Itertools;
use std::ops::Range;

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord>;

    fn bounding_box(&self) -> &AABB;
}

pub struct HitRecord<'a> {
    pub point: DVec3,
    pub normal: DVec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    /// NOTE: the parameter `outward_normal` is assumed to have unit length.
    pub fn with_face_normal(
        point: DVec3,
        outward_normal: DVec3,
        t: f64,
        u: f64,
        v: f64,
        ray: &Ray,
        material: &'a dyn Material,
    ) -> Self {
        let (front_face, normal) = Self::calc_face_normal(ray, outward_normal);

        Self {
            point,
            normal,
            t,
            u,
            v,
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

pub struct Translate<T>
where
    T: Hittable,
{
    object: T,
    offset: DVec3,
    bbox: AABB,
}

impl<T: Hittable> Translate<T> {
    pub fn new(object: T, displacement: DVec3) -> Self {
        Self {
            bbox: object.bounding_box().offset(displacement),
            object,
            offset: displacement,
        }
    }
}

impl<T: Hittable> Hittable for Translate<T> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        let offset_ray = Ray::new(ray.orig - self.offset, ray.dir);

        self.object.hit(&offset_ray, interval).map(|mut hit| {
            hit.point += self.offset;
            hit
        })
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}

pub struct RotateY<T>
where
    T: Hittable,
{
    object: T,
    sin_theta: f64,
    cos_theta: f64,
    bbox: AABB,
}

impl<T> RotateY<T>
where
    T: Hittable,
{
    pub fn new(object: T, angle: f64) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box();

        let (min, max) = (0..2)
            .cartesian_product(0..2)
            .cartesian_product(0..2)
            .map(|((i, j), k)| {
                let new_point = dvec3(i as f64, j as f64, k as f64) * bbox.max
                    + dvec3((1 - i) as f64, (1 - j) as f64, (1 - k) as f64) * bbox.min;

                let newx = cos_theta * new_point.x + sin_theta * new_point.z;
                let newz = -sin_theta * new_point.x + cos_theta * new_point.z;

                dvec3(newx, new_point.y, newz)
            })
            .fold(
                (DVec3::INFINITY, DVec3::NEG_INFINITY),
                |(min, max), tester| (min.min(tester), max.max(tester)),
            );

        Self {
            object,
            sin_theta,
            cos_theta,
            bbox: AABB::new(min, max),
        }
    }
}

impl<T: Hittable> Hittable for RotateY<T> {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        let mut origin = ray.orig;
        let mut dir = ray.dir;

        origin[0] = self.cos_theta * ray.orig[0] - self.sin_theta * ray.orig[2];
        origin[2] = self.sin_theta * ray.orig[0] + self.cos_theta * ray.orig[2];

        dir[0] = self.cos_theta * ray.dir[0] - self.sin_theta * ray.dir[2];
        dir[2] = self.sin_theta * ray.dir[0] + self.cos_theta * ray.dir[2];

        let rotated_r = Ray::new(origin, dir);

        // Determine where (if any) an intersection occurs in object space
        self.object.hit(&rotated_r, interval).map(|mut hit| {
            // Change the intersection point from object space to world space
            let mut p = hit.point;
            p[0] = self.cos_theta * hit.point[0] + self.sin_theta * hit.point[2];
            p[2] = -self.sin_theta * hit.point[0] + self.cos_theta * hit.point[2];

            // Change the normal from object space to world space
            let mut normal = hit.normal;
            normal[0] = self.cos_theta * hit.normal[0] + self.sin_theta * hit.normal[2];
            normal[2] = -self.sin_theta * hit.normal[0] + self.cos_theta * hit.normal[2];

            hit.point = p;
            hit.normal = normal;
            hit
        })
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
