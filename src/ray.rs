use crate::{color::Color, hittable::Hittable};
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

    pub fn color(&self, depth: u32, world: &impl Hittable) -> Color {
        if depth == 0 {
            return Color::ZERO;
        }

        if let Some(hit) = world.hit(self, &(0.001..f64::INFINITY)) {
            if let Some(scatter) = hit.material.scatter(self, &hit) {
                return scatter.attenuation * scatter.ray.color(depth - 1, world);
            }
            return Color::ZERO;
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
    }
}
