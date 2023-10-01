use glam::{dvec3, DVec3};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rusticrayz::camera::{Camera, CameraSettings};
use rusticrayz::color::Color;
use rusticrayz::material::*;
use rusticrayz::sphere::Sphere;
use std::{io, sync::Arc};

fn main() -> io::Result<()> {
    let material_ground = Arc::new(Lambertian {
        albedo: Color::new(0.5, 0.5, 0.5),
    });

    let mut world = Vec::new();
    world.push(Sphere::new(dvec3(0., -1000., 0.), 1000., material_ground));

    let mut rng = thread_rng();
    for (a, b) in (-11..11).cartesian_product(-11..11) {
        let choose_mat = rng.gen::<f64>();
        let center = dvec3(
            a as f64 + 0.9 * rng.gen::<f64>(),
            0.2,
            b as f64 + 0.9 * rng.gen::<f64>(),
        );

        if (center - dvec3(4., 0.2, 0.)).length_squared() > 0.9 * 0.9 {
            if choose_mat < 0.8 {
                let mat = Arc::new(Lambertian { albedo: rng.gen() });
                world.push(Sphere::new(center, 0.2, mat));
            } else if choose_mat < 0.95 {
                let mat = Arc::new(Metallic {
                    albedo: rng.gen(),
                    fuzz: rng.gen_range(0.0..0.5),
                });
                world.push(Sphere::new(center, 0.2, mat));
            } else {
                let mat = Arc::new(Dielectric {
                    refraction_idx: 1.5,
                });
                world.push(Sphere::new(center, 0.2, mat));
            };
        }
    }

    let mat1 = Arc::new(Dielectric {
        refraction_idx: 1.5,
    });
    world.push(Sphere::new(DVec3::Y, 1., mat1));

    let mat2 = Arc::new(Lambertian {
        albedo: dvec3(0.4, 0.2, 0.1),
    });
    world.push(Sphere::new(dvec3(-4., 1., 0.), 1., mat2));

    let mat3 = Arc::new(Metallic {
        albedo: dvec3(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    world.push(Sphere::new(dvec3(4., 1., 0.), 1., mat3));

    let camera = Camera::new(CameraSettings {
        image_width: 1200,
        aspect_ratio: 16. / 9.,
        samples_per_pixel: 10,
        max_depth: 50,
        look_from: Some(dvec3(13., 2., 3.)),
        look_at: Some(dvec3(0., 0., 0.)),
        view_up: Some(DVec3::Y),
        vfov: Some(20.),
        defocus_angle: Some(0.0),
        focus_dist: Some(10.),
    });
    camera.render_to_disk(&world)?;
    Ok(())
}
