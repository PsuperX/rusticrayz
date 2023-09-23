use crate::{hittable::Hittable, ray::Ray};
use glam::{dvec3, DVec3};
use indicatif::ProgressIterator;
use itertools::Itertools;
use std::{fs, io};

pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    center: DVec3,
    pixel00_loc: DVec3,
    pixel_delta_v: DVec3,
    pixel_delta_u: DVec3,
}

impl Camera {
    pub fn new(image_width: u32, aspect_ratio: f64) -> Self {
        let image_height = if ((image_width as f64 / aspect_ratio) as u32) < 1 {
            1
        } else {
            (image_width as f64 / aspect_ratio) as u32
        };

        let focal_lenght = 1.;
        let viewport_height = 2.;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
        let center = DVec3::ZERO;

        let viewport_u = dvec3(viewport_width, 0., 0.);
        let viewport_v = dvec3(0., -viewport_height, 0.);
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            center - dvec3(0., 0., focal_lenght) - viewport_u / 2. - viewport_v / 2.;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_v,
            pixel_delta_u,
        }
    }

    pub fn render_to_disk(&self, world: &impl Hittable) -> io::Result<()> {
        let pixels = (0..self.image_height)
            .cartesian_product(0..self.image_width)
            .progress_count(self.image_width as u64 * self.image_height as u64)
            .map(|(y, x)| {
                let pixel_center = self.pixel00_loc
                    + (x as f64 * self.pixel_delta_u)
                    + (y as f64 * self.pixel_delta_v);
                let ray_dir = pixel_center - self.center;

                let ray = Ray::new(self.center, ray_dir);
                let pixel_color = ray.color(world) * 255.;
                format!("{} {} {}", pixel_color.x, pixel_color.y, pixel_color.z)
            })
            .join(" ");
        fs::write(
            "image.ppm",
            format!("P3 {} {} 255 {pixels}", self.image_width, self.image_height),
        )?;

        Ok(())
    }
}
