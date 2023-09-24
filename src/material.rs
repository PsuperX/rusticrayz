use crate::{color::Color, hittable::HitRecord, ray::Ray, vectors::Dvec3Extensions};
use glam::DVec3;

pub trait Material {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scattered>;
}

pub struct Scattered {
    pub ray: Ray,
    pub attenuation: Color,
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<Scattered> {
        let mut scatter_dir = hit.normal + DVec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }

        Some(Scattered {
            ray: Ray::new(hit.point, scatter_dir),
            attenuation: self.albedo,
        })
    }
}

pub struct Metallic {
    pub albedo: Color,
}

impl Material for Metallic {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scattered> {
        let reflected = ray.dir.normalize().reflect(hit.normal);
        Some(Scattered {
            ray: Ray::new(hit.point, reflected),
            attenuation: self.albedo,
        })
    }
}
