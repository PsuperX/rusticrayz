use crate::color::Color;
use glam::{dvec3, DVec3};

pub struct Ray {
    pub orig: DVec3,
    pub dir: DVec3,
}

impl Ray {
    pub fn new(orig: DVec3, dir: DVec3) -> Self {
        Self { orig, dir }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.orig + self.dir * t
    }

    pub fn color(&self) -> Color {
        let sphere_center = dvec3(0., 0., -1.);
        let t = self.hit_sphere(sphere_center, 0.5);
        if let Some(t) = t {
            let normal = self.at(t) - sphere_center;
            return 0.5 * Color::new(normal.x + 1., normal.y + 1., normal.z + 1.);
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
    }

    pub fn hit_sphere(&self, center: DVec3, radius: f64) -> Option<f64> {
        let ac = self.orig - center;
        let a = self.dir.length_squared();
        let half_b = self.dir.dot(ac);
        let c = ac.length_squared() - radius * radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0. {
            None
        } else {
            Some((-half_b - discriminant.sqrt()) / a)
        }
    }
}
