use crate::{hittable::Hittable, ray::Ray};
use glam::{dvec3, DVec3};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{thread_rng, Rng};
use std::{fs, io};

pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    center: DVec3,
    pixel00_loc: DVec3,
    pixel_delta_v: DVec3,
    pixel_delta_u: DVec3,
    samples_per_pixel: u32,
    max_depth: u32,
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
            samples_per_pixel: 100,
            max_depth: 50,
        }
    }

    pub fn render_to_disk(&self, world: &impl Hittable) -> io::Result<()> {
        let pixels = (0..self.image_height)
            .cartesian_product(0..self.image_width)
            .progress_count(self.image_width as u64 * self.image_height as u64)
            .map(|(y, x)| {
                let multisampled_pixel_color = (0..self.samples_per_pixel)
                    .map(|_| self.get_ray(x, y).color(self.max_depth, world))
                    .sum::<DVec3>();

                let scale = 1. / self.samples_per_pixel as f64;
                let pixel_color = 255. * scale * multisampled_pixel_color;
                format!("{} {} {}", pixel_color.x, pixel_color.y, pixel_color.z)
            })
            .join(" ");
        fs::write(
            "image.ppm",
            format!("P3 {} {} 255 {pixels}", self.image_width, self.image_height),
        )?;

        Ok(())
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        let pixel_center =
            self.pixel00_loc + (x as f64 * self.pixel_delta_u) + (y as f64 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.pixel_sample_square();
        let ray_dir = pixel_sample - self.center;

        Ray::new(self.center, ray_dir)
    }

    fn pixel_sample_square(&self) -> DVec3 {
        let mut rng = thread_rng();
        let px = -0.5 + rng.gen::<f64>();
        let py = -0.5 + rng.gen::<f64>();
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }
}
