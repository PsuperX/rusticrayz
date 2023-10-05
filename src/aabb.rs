use crate::ray::Ray;
use glam::DVec3;
use std::{mem, ops::Range};

#[derive(Debug, Default, Clone)]
pub struct AABB {
    x: Range<f64>,
    y: Range<f64>,
    z: Range<f64>,
}

impl AABB {
    pub fn new(a: DVec3, b: DVec3) -> Self {
        Self {
            x: a.x.min(b.x)..a.x.max(b.x),
            y: a.y.min(b.y)..a.y.max(b.y),
            z: a.z.min(b.z)..a.z.max(b.z),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            x: merge_range(&self.x, &other.x),
            y: merge_range(&self.y, &other.y),
            z: merge_range(&self.z, &other.z),
        }
    }

    pub fn axis(&self, n: usize) -> &Range<f64> {
        match n {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("no such axis {n}"),
        }
    }

    pub fn hit(&self, ray: &Ray, interval: &Range<f64>) -> bool {
        for a in 0..3 {
            let inv_d = ray.dir[a].recip();
            let orig = ray.orig[a];

            let mut t0 = (self.axis(a).start - orig) * inv_d;
            let mut t1 = (self.axis(a).end - orig) * inv_d;

            if inv_d < 0. {
                mem::swap(&mut t0, &mut t1);
            }

            let interval = t0.max(interval.start)..t1.min(interval.end);
            if interval.end <= interval.start {
                return false;
            }
        }
        true
    }
}

fn merge_range(a: &Range<f64>, b: &Range<f64>) -> Range<f64> {
    a.start.min(b.start)..(a.end.max(b.end))
}
