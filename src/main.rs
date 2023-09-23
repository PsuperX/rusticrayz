mod camera;
mod color;
mod hittable;
mod ray;
mod sphere;

use camera::Camera;
use glam::dvec3;
use hittable::HittableList;
use sphere::Sphere;
use std::io;

const ASPECT_RATIO: f64 = 16. / 9.;
const IMAGE_WIDTH: u32 = 400;

fn main() -> io::Result<()> {
    let world = HittableList {
        objects: vec![
            Box::new(Sphere::new(dvec3(0., 0., -1.), 0.5)),
            Box::new(Sphere::new(dvec3(0., -100.5, -1.), 100.)),
        ],
    };

    let camera = Camera::new(IMAGE_WIDTH, ASPECT_RATIO);
    camera.render_to_disk(&world)?;
    Ok(())
}
