use crate::ray::Ray;
use glam::DVec3;
use std::{mem, ops::Range};

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: DVec3,
    pub max: DVec3,
}

impl AABB {
    /// An empty `Aabb` should not contain any point
    pub fn empty() -> Self {
        Self {
            min: DVec3::splat(f64::INFINITY),
            max: DVec3::splat(f64::NEG_INFINITY),
        }
    }

    pub fn new(a: DVec3, b: DVec3) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn grow(&self, other: DVec3) -> Self {
        Self {
            min: self.min.min(other),
            max: self.max.max(other),
        }
    }

    pub fn pad(&self) -> Self {
        let delta = 0.0001;
        Self {
            min: self.min - delta,
            max: self.max + delta,
        }
    }

    pub fn contains(&self, point: &DVec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn hit(&self, ray: &Ray, interval: &Range<f64>) -> bool {
        for a in 0..3 {
            let inv_d = ray.dir[a].recip();
            let orig = ray.orig[a];

            let mut t0 = (self.min[a] - orig) * inv_d;
            let mut t1 = (self.max[a] - orig) * inv_d;

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

    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    pub fn center(&self) -> DVec3 {
        self.min + self.size() * 0.5
    }

    pub fn surface_area(&self) -> f64 {
        let size = self.size();
        2.0 * size.dot(size)
    }

    pub fn is_empty(&self) -> bool {
        self.min.max(self.max) != self.max
    }

    pub fn largest_axis(&self) -> usize {
        let size = self.size();
        let largest = size.max_element();

        if largest == size.x {
            0
        } else if largest == size.y {
            1
        } else {
            2
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::empty()
    }
}
