use glam::{dvec3, DVec3};
use std::{ops::Range, sync::Arc};

use super::Quad;
use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
};

pub struct QuadBox {
    objects: [Quad; 6],
    bbox: AABB,
}

impl QuadBox {
    /// Returns the 3D box (six sides) that contains the two opposite vertices a & b.
    pub fn new(a: DVec3, b: DVec3, mat: Arc<dyn Material + Send + Sync>) -> Self {
        // Construct the two opposite vertices with the minimum and maximum coordinates.
        let min = dvec3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let max = dvec3(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

        let dx = dvec3(max.x - min.x, 0.0, 0.0);
        let dy = dvec3(0.0, max.y - min.y, 0.0);
        let dz = dvec3(0.0, 0.0, max.z - min.z);

        let sides = [
            Quad::new(dvec3(min.x, min.y, max.z), dx, dy, mat.clone()), // front
            Quad::new(dvec3(max.x, min.y, max.z), -dz, dy, mat.clone()), // right
            Quad::new(dvec3(max.x, min.y, min.z), -dx, dy, mat.clone()), // back
            Quad::new(dvec3(min.x, min.y, min.z), dz, dy, mat.clone()), // left
            Quad::new(dvec3(min.x, max.y, max.z), dx, -dz, mat.clone()), // top
            Quad::new(dvec3(min.x, min.y, min.z), dx, dz, mat),         // bottom
        ];

        Self {
            objects: sides,
            bbox: AABB::new(min, max),
        }
    }
}

impl Hittable for QuadBox {
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
