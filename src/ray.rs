use crate::{color::Color, hittable::Hittable};
use glam::DVec3;
use rand::{thread_rng, Rng};

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

        if let Some(hit) = world.hit(self, f64::EPSILON..f64::INFINITY) {
            let dir = hit.normal + random_in_unit_sphere();
            let ray = Ray::new(hit.point, dir);
            return 0.5 * ray.color(depth - 1, world);
        }

        let unit_dir = self.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.);
        Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
    }
}

fn random_in_unit_sphere() -> DVec3 {
    let mut rng = thread_rng();
    loop {
        let p = DVec3::new(
            rng.gen_range((-1.)..1.),
            rng.gen_range((-1.)..1.),
            rng.gen_range((-1.)..1.),
        );

        if p.length_squared() < 1. {
            return p;
        }
    }
}

fn random_unit_vector() -> DVec3 {
    random_in_unit_sphere().normalize()
}

fn random_on_hemisphere(normal: DVec3) -> DVec3 {
    let on_unit_sphere = random_unit_vector();
    if on_unit_sphere.dot(normal) > 0. {
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}
