use crate::{
    color::Color,
    hittable::HitRecord,
    ray::Ray,
    texture::{SolidColor, Texture},
    vectors::{reflectance, Dvec3Extensions},
};
use glam::DVec3;
use rand::{thread_rng, Rng};

pub trait Material {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scattered>;

    fn emitted(&self, _u: f64, _v: f64, _point: Color) -> Color {
        Color::ZERO
    }
}

pub struct Scattered {
    pub ray: Ray,
    pub attenuation: Color,
}

#[derive(Clone)]
pub struct Lambertian<T>
where
    T: Texture,
{
    pub albedo: T,
}

impl<T> Lambertian<T>
where
    T: Texture,
{
    pub fn new(albedo: T) -> Self {
        Self { albedo }
    }
}

impl Lambertian<SolidColor> {
    pub fn from_color(albedo: Color) -> Self {
        Self {
            albedo: SolidColor::from(albedo),
        }
    }
}

impl<T> Material for Lambertian<T>
where
    T: Texture,
{
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<Scattered> {
        let mut scatter_dir = hit.normal + DVec3::random_unit_vector();

        // Catch degenerate scatter direction
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }

        Some(Scattered {
            ray: Ray::new(hit.point, scatter_dir),
            attenuation: self.albedo.color(hit.u, hit.v, hit.point),
        })
    }
}

#[derive(Clone)]
pub struct Metallic {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metallic {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scattered> {
        let reflected = ray.dir.normalize().reflect(hit.normal);
        Some(Scattered {
            ray: Ray::new(
                hit.point,
                reflected + self.fuzz * DVec3::random_unit_vector(),
            ),
            attenuation: self.albedo,
        })
    }
}

#[derive(Clone)]
pub struct Dielectric {
    pub refraction_idx: f64,
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<Scattered> {
        let mut rng = thread_rng();
        let refraction_ratio = if hit.front_face {
            1. / self.refraction_idx
        } else {
            self.refraction_idx
        };

        let unit_dir = ray.dir.normalize();
        let cos_theta = (-unit_dir).dot(hit.normal).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.;
        let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.gen() {
            unit_dir.reflect(hit.normal)
        } else {
            unit_dir.refract(hit.normal, refraction_ratio)
        };

        Some(Scattered {
            ray: Ray::new(hit.point, direction),
            attenuation: Color::ONE,
        })
    }
}

pub struct DiffuseLight<T>
where
    T: Texture,
{
    emit: T,
}

impl<T> DiffuseLight<T>
where
    T: Texture,
{
    pub fn new(emit: T) -> Self {
        Self { emit }
    }
}

impl DiffuseLight<SolidColor> {
    pub fn from_color(emit: Color) -> Self {
        Self { emit: emit.into() }
    }
}

impl<T> Material for DiffuseLight<T>
where
    T: Texture,
{
    fn scatter(&self, _ray: &Ray, _hit: &HitRecord) -> Option<Scattered> {
        None
    }

    fn emitted(&self, u: f64, v: f64, point: Color) -> Color {
        self.emit.color(u, v, point)
    }
}
