mod color;
mod ray;

use glam::{dvec3, DVec3};
use indicatif::ProgressIterator;
use itertools::Itertools;
use ray::Ray;
use std::fs;

const ASPECT_RATIO: f64 = 16. / 9.;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = if ((IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32) < 1 {
    1
} else {
    (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32
};

const FOCAL_LENGHT: f64 = 1.;
const VIEWPORT_HEIGHT: f64 = 2.;
const VIEWPORT_WIDTH: f64 = VIEWPORT_HEIGHT * (IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64);
const CAMERA_CENTER: DVec3 = DVec3::ZERO;

const VIEWPORT_U: DVec3 = dvec3(VIEWPORT_WIDTH, 0., 0.);
const VIEWPORT_V: DVec3 = dvec3(0., -VIEWPORT_HEIGHT, 0.);

fn main() -> Result<(), std::io::Error> {
    let pixel_delta_u = VIEWPORT_U / IMAGE_WIDTH as f64;
    let pixel_delta_v = VIEWPORT_V / IMAGE_HEIGHT as f64;

    let viewport_upper_left =
        CAMERA_CENTER - dvec3(0., 0., FOCAL_LENGHT) - VIEWPORT_U / 2. - VIEWPORT_V / 2.;

    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let pixels = (0..IMAGE_HEIGHT)
        .cartesian_product(0..IMAGE_WIDTH)
        .progress_count(IMAGE_WIDTH as u64 * IMAGE_HEIGHT as u64)
        .map(|(y, x)| {
            let pixel_center =
                pixel00_loc + (x as f64 * pixel_delta_u) + (y as f64 * pixel_delta_v);
            let ray_dir = pixel_center - CAMERA_CENTER;

            let ray = Ray::new(CAMERA_CENTER, ray_dir);
            let pixel_color = ray.color() * 255.;
            format!("{} {} {}", pixel_color.x, pixel_color.y, pixel_color.z)
        })
        .join(" ");
    fs::write(
        "image.ppm",
        format!("P3 {IMAGE_WIDTH} {IMAGE_HEIGHT} 255 {pixels}"),
    )?;
    Ok(())
}
