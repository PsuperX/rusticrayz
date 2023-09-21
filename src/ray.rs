use crate::color::Color;
use glam::DVec3;

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
        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
    }
}
