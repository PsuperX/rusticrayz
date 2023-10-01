mod camera;
mod color;
mod hittable;
mod material;
mod ray;
mod sphere;
mod vectors;

use camera::Camera;
use color::Color;
use glam::{dvec3, DVec3};
use hittable::HittableList;
use material::*;
use sphere::Sphere;
use std::{io, rc::Rc};

const ASPECT_RATIO: f64 = 16. / 9.;
const IMAGE_WIDTH: u32 = 400;

fn main() -> io::Result<()> {
    let material_ground = Rc::new(Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    });
    let material_center = Rc::new(Lambertian {
        albedo: Color::new(0.1, 0.2, 0.5),
    });
    let material_left = Rc::new(Dielectric {
        refraction_idx: 1.5,
    });
    let material_right = Rc::new(Metallic {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzz: 0.0,
    });

    let world = HittableList {
        objects: vec![
            Box::new(Sphere::new(
                dvec3(0.0, -100.5, -1.0),
                100.0,
                material_ground,
            )),
            Box::new(Sphere::new(dvec3(0.0, 0.0, -1.0), 0.5, material_center)),
            Box::new(Sphere::new(
                dvec3(-1.0, 0.0, -1.0),
                0.5,
                material_left.clone(),
            )),
            Box::new(Sphere::new(dvec3(-1.0, 0.0, -1.0), -0.4, material_left)),
            Box::new(Sphere::new(dvec3(1.0, 0.0, -1.0), 0.5, material_right)),
        ],
    };

    let look_from = Some(dvec3(-2.0, 2.0, 1.0));
    let look_at = Some(dvec3(0.0, 0.0, -1.0));
    let view_up = Some(DVec3::Y);
    let camera = Camera::new(
        IMAGE_WIDTH,
        ASPECT_RATIO,
        look_from,
        look_at,
        view_up,
        Some(20.),
        Some(10.),
        Some(3.4),
    );
    camera.render_to_disk(&world)?;
    Ok(())
}
