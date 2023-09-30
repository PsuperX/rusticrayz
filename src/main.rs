mod camera;
mod color;
mod hittable;
mod material;
mod ray;
mod sphere;
mod vectors;

use camera::Camera;
use color::Color;
use glam::dvec3;
use hittable::HittableList;
use material::*;
use sphere::Sphere;
use std::{f64::consts::PI, io, rc::Rc};

const ASPECT_RATIO: f64 = 16. / 9.;
const IMAGE_WIDTH: u32 = 400;

fn main() -> io::Result<()> {
    let r = (PI / 4.).cos();
    let material_left = Rc::new(Lambertian {
        albedo: Color::new(0., 0., 1.0),
    });
    let material_right = Rc::new(Lambertian {
        albedo: Color::new(1.0, 0.0, 0.0),
    });

    let world = HittableList {
        objects: vec![
            Box::new(Sphere::new(dvec3(-r, 0.0, -1.0), r, material_left)),
            Box::new(Sphere::new(dvec3(r, 0.0, -1.0), r, material_right)),
        ],
    };

    let camera = Camera::new(IMAGE_WIDTH, ASPECT_RATIO);
    camera.render_to_disk(&world)?;
    Ok(())
}
