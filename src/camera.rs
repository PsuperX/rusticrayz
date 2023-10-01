use crate::{hittable::Hittable, ray::Ray, vectors::Dvec3Extensions};
use glam::{dvec3, DVec3};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{thread_rng, Rng};
use std::{fs, io};

pub struct CameraSettings {
    pub image_width: u32,
    pub aspect_ratio: f64,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub look_from: Option<DVec3>,
    pub look_at: Option<DVec3>,
    pub view_up: Option<DVec3>,
    pub vfov: Option<f64>,
    pub defocus_angle: Option<f64>,
    pub focus_dist: Option<f64>,
}

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
    defocus_angle: f64,
    defocus_disk_u: DVec3,
    defocus_disk_v: DVec3,
}

impl Camera {
    pub fn new(settings: CameraSettings) -> Self {
        let image_width = settings.image_width;
        let aspect_ratio = settings.aspect_ratio;
        let look_from = settings.look_from.unwrap_or(DVec3::NEG_Z);
        let look_at = settings.look_at.unwrap_or(DVec3::ZERO);
        let view_up = settings.view_up.unwrap_or(DVec3::Y);
        let vfov = settings.vfov.unwrap_or(20.);
        let defocus_angle = settings.defocus_angle.unwrap_or(0.);
        let focus_dist = settings.focus_dist.unwrap_or(10.);

        let image_height = if ((image_width as f64 / aspect_ratio) as u32) < 1 {
            1
        } else {
            (image_width as f64 / aspect_ratio) as u32
        };

        let center = look_from;

        // Viewport dimensions
        let theta = vfov.to_radians();
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h * focus_dist;
        let viewport_width = viewport_height * (image_width as f64 / image_height as f64);

        let w = (look_from - look_at).normalize();
        let u = view_up.cross(w).normalize();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = -viewport_height * v;
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left = center - (focus_dist * w) - viewport_u / 2. - viewport_v / 2.;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = focus_dist * (defocus_angle / 2.).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_v,
            pixel_delta_u,
            samples_per_pixel: settings.samples_per_pixel,
            max_depth: settings.max_depth,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
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
                let mut pixel_color = scale * multisampled_pixel_color;
                pixel_color = 255.
                    * linear_to_gamma(pixel_color).clamp(DVec3::splat(0.), DVec3::splat(0.999));
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

        let ray_orig = if self.defocus_angle <= 0. {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_dir = pixel_sample - self.center;

        Ray::new(ray_orig, ray_dir)
    }

    /// Returns a random point in the camera defocus disk
    fn defocus_disk_sample(&self) -> DVec3 {
        let p = DVec3::random_in_unit_disk();
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn pixel_sample_square(&self) -> DVec3 {
        let mut rng = thread_rng();
        let px = -0.5 + rng.gen::<f64>();
        let py = -0.5 + rng.gen::<f64>();
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }
}

fn linear_to_gamma(linear: DVec3) -> DVec3 {
    dvec3(linear.x.sqrt(), linear.y.sqrt(), linear.z.sqrt())
}
