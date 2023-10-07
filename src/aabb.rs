use crate::ray::Ray;
use glam::{dvec3, DVec3};
use std::{cmp::Ordering, mem, ops::Range};

#[derive(Debug, Clone)]
pub struct AABB {
    x: Range<f64>,
    y: Range<f64>,
    z: Range<f64>,
}

impl AABB {
    /// An empty `Aabb` should not contain any point
    pub fn empty() -> Self {
        Self {
            x: f64::INFINITY..f64::NEG_INFINITY,
            y: f64::INFINITY..f64::NEG_INFINITY,
            z: f64::INFINITY..f64::NEG_INFINITY,
        }
    }

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

    pub fn grow(&self, other: &DVec3) -> Self {
        Self {
            x: self.x.start.min(other.x)..self.x.end.max(other.x),
            y: self.y.start.min(other.y)..self.y.end.max(other.y),
            z: self.z.start.min(other.z)..self.z.end.max(other.z),
        }
    }

    pub fn contains(&self, point: &DVec3) -> bool {
        self.x.contains(&point.x) && self.y.contains(&point.y) && self.z.contains(&point.z)
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

    pub fn center(&self) -> DVec3 {
        dvec3(
            (self.x.end + self.x.start) / 2.,
            (self.y.end + self.y.start) / 2.,
            (self.z.end + self.z.start) / 2.,
        )
    }

    pub fn surface_area(&self) -> f64 {
        let size = dvec3(self.x.end, self.y.end, self.x.end)
            - dvec3(self.x.start, self.y.start, self.x.start);
        2. * size.dot(size)
    }

    pub fn is_empty(&self) -> bool {
        self.x.is_empty() && self.y.is_empty() && self.z.is_empty()
    }

    pub fn largest_axis(&self) -> usize {
        // Safety: Array is always of lenght 3
        unsafe {
            [
                self.x.end - self.x.start,
                self.y.end - self.y.start,
                self.z.end - self.z.start,
            ]
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_unchecked()
            .0
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::empty()
    }
}

fn merge_range(a: &Range<f64>, b: &Range<f64>) -> Range<f64> {
    a.start.min(b.start)..(a.end.max(b.end))
}
