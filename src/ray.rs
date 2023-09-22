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
        if self.hit_sphere(dvec3(0., 0., -1.), 0.5) {
            return Color::new(1., 0., 0.);
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
    }

    pub fn hit_sphere(&self, center: DVec3, radius: f64) -> bool {
        let ac = self.orig - center;
        let a = self.dir.dot(self.dir);
        let b = 2. * self.dir.dot(ac);
        let c = ac.dot(ac) - radius * radius;
        let discriminant = b * b - 4. * a * c;
        discriminant >= 0.
    }
}
