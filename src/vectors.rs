use glam::DVec3;
use rand::{thread_rng, Rng};

pub trait Dvec3Extensions {
    fn random_in_unit_sphere() -> DVec3;
    fn random_unit_vector() -> DVec3;
    fn reflect(self, n: DVec3) -> DVec3;
    fn near_zero(&self) -> bool;
}

impl Dvec3Extensions for DVec3 {
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
        Self::random_in_unit_sphere().normalize()
    }

    fn reflect(self, n: DVec3) -> DVec3 {
        self - 2. * self.dot(n) * n
    }

    fn near_zero(&self) -> bool {
        self.abs_diff_eq(DVec3::ZERO, 1e-8)
    }
}

pub fn random_on_hemisphere(normal: DVec3) -> DVec3 {
    let on_unit_sphere = DVec3::random_unit_vector();
    if on_unit_sphere.dot(normal) > 0. {
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}
