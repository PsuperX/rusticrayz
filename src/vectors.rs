use glam::DVec3;
use rand::{thread_rng, Rng};

pub trait Dvec3Extensions {
    fn random_in_unit_sphere() -> Self;
    fn random_unit_vector() -> Self;
    fn reflect(self, n: Self) -> Self;
    fn refract(self, normal: Self, etai_over_etat: f64) -> Self;
    fn near_zero(&self) -> bool;
}

impl Dvec3Extensions for DVec3 {
    fn random_in_unit_sphere() -> Self {
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

    fn random_unit_vector() -> Self {
        Self::random_in_unit_sphere().normalize()
    }

    fn reflect(self, n: Self) -> Self {
        self - 2. * self.dot(n) * n
    }

    fn refract(self, normal: Self, etai_over_etat: f64) -> Self {
        let cos_theta = (-self).dot(normal).min(1.);
        let r_out_perp = etai_over_etat * (self + cos_theta * normal);
        let r_out_parallel = -((1. - r_out_perp.length_squared()).abs()).sqrt() * normal;
        r_out_perp + r_out_parallel
    }

    fn near_zero(&self) -> bool {
        self.abs_diff_eq(Self::ZERO, 1e-8)
    }
}

pub fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let mut r0 = (1. - ref_idx) / (1. + ref_idx);
    r0 = r0 * r0;
    r0 + (1. - r0) * (1. - cosine).powi(5)
}

pub fn random_on_hemisphere(normal: DVec3) -> DVec3 {
    let on_unit_sphere = DVec3::random_unit_vector();
    if on_unit_sphere.dot(normal) > 0. {
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}
