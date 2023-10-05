use crate::{
    aabb::AABB,
    hittable::{HitRecord, Hittable},
    ray::Ray,
};
use rand::{thread_rng, Rng};
use std::{cmp::Ordering, ops::Range};

#[derive(Clone)]
pub struct BvhNode {
    left: Box<dyn Hittable + Sync>,
    right: Box<dyn Hittable + Sync>,
    bbox: AABB,
}

impl BvhNode {
    // TODO: tooooo much cloning going on
    pub fn new(mut objects: Vec<Box<dyn Hittable + Sync>>) -> Self {
        let mut rng = thread_rng();
        let axis = rng.gen_range(0..=2);
        let comparator = match axis {
            0 => box_x_compare,
            1 => box_y_compare,
            2 => box_z_compare,
            _ => unreachable!(),
        };

        let object_span = objects.len();
        let (left, right) = match object_span {
            1 => (objects[0].clone(), objects[0].clone()),
            2 => {
                if comparator(&objects[0], objects.last().unwrap()).is_lt() {
                    (objects[0].clone(), objects.last().unwrap().clone())
                } else {
                    (objects.last().unwrap().clone(), objects[0].clone())
                }
            }
            _ => {
                objects.sort_unstable_by(comparator);

                let mid = object_span / 2;
                let right = objects.split_off(mid);
                (
                    Box::new(BvhNode::new(objects)) as Box<dyn Hittable + Sync>,
                    Box::new(BvhNode::new(right)) as Box<dyn Hittable + Sync>,
                )
            }
        };

        Self {
            bbox: left.bounding_box().merge(right.bounding_box()),
            left,
            right,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, interval: &Range<f64>) -> Option<HitRecord> {
        if !self.bbox.hit(ray, interval) {
            return None;
        }

        let hit_left = self.left.hit(ray, interval);
        let hit_right = self.right.hit(
            ray,
            &(interval.start..hit_left.as_ref().map_or(interval.end, |hit| hit.t)),
        );

        hit_right.or(hit_left)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}

fn box_compare(
    a: &(dyn Hittable + Sync),
    b: &(dyn Hittable + Sync),
    axis_index: usize,
) -> Ordering {
    a.bounding_box()
        .axis(axis_index)
        .start
        .total_cmp(&b.bounding_box().axis(axis_index).start)
}

// TODO: remove box?
fn box_x_compare(a: &Box<dyn Hittable + Sync>, b: &Box<dyn Hittable + Sync>) -> Ordering {
    box_compare(a.as_ref(), b.as_ref(), 0)
}
fn box_y_compare(a: &Box<dyn Hittable + Sync>, b: &Box<dyn Hittable + Sync>) -> Ordering {
    box_compare(a.as_ref(), b.as_ref(), 1)
}
fn box_z_compare(a: &Box<dyn Hittable + Sync>, b: &Box<dyn Hittable + Sync>) -> Ordering {
    box_compare(a.as_ref(), b.as_ref(), 2)
}
