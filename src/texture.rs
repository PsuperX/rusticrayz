use crate::color::Color;
use glam::{dvec3, DVec3};
use image::{DynamicImage, GenericImageView};
use noise::{NoiseFn, Perlin};
use std::path::Path;

pub trait Texture {
    fn color(&self, u: f64, v: f64, point: DVec3) -> Color;
}

pub struct SolidColor {
    pub color: Color,
}

impl Texture for SolidColor {
    fn color(&self, _u: f64, _v: f64, _point: DVec3) -> Color {
        self.color
    }
}

impl From<Color> for SolidColor {
    fn from(value: Color) -> Self {
        Self { color: value }
    }
}

pub struct CheckerTexture<E, O>
where
    E: Texture,
    O: Texture,
{
    inv_scale: f64,
    even: E,
    odd: O,
}

impl<E, O> CheckerTexture<E, O>
where
    E: Texture,
    O: Texture,
{
    pub fn new(scale: f64, even_color: E, odd_color: O) -> Self {
        Self {
            inv_scale: scale.recip(),
            even: even_color,
            odd: odd_color,
        }
    }
}

impl<E, O> Texture for CheckerTexture<E, O>
where
    E: Texture,
    O: Texture,
{
    fn color(&self, u: f64, v: f64, point: DVec3) -> Color {
        let x_int = (self.inv_scale * point.x).floor() as i32;
        let y_int = (self.inv_scale * point.y).floor() as i32;
        let z_int = (self.inv_scale * point.z).floor() as i32;

        let is_even = (x_int + y_int + z_int) % 2 == 0;

        if is_even {
            self.even.color(u, v, point)
        } else {
            self.odd.color(u, v, point)
        }
    }
}

pub struct ImageTexture {
    image: DynamicImage,
}

impl ImageTexture {
    pub fn load_image(path: impl AsRef<Path>) -> image::ImageResult<Self> {
        let image = image::open(path)?;
        Ok(Self { image })
    }
}

impl Texture for ImageTexture {
    fn color(&self, u: f64, v: f64, _point: DVec3) -> Color {
        // If we have no texture data, then return solid cyan as a debugging aid.
        if self.image.height() == 0 {
            return dvec3(0., 1., 1.);
        }

        // Clamp input texture coorenates to [0,1] x [1,0]
        let u = u.clamp(0., 1.);
        let v = 1. - v.clamp(0., 1.);

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let pixel = self.image.get_pixel(i, j);

        let color_scale = 1. / 255.;
        dvec3(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64) * color_scale
    }
}

pub struct NoiseTexture {
    noise: noise::Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64, seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn color(&self, _u: f64, _v: f64, point: DVec3) -> Color {
        (1.0 + self.noise.get((self.scale * point).into())) * 0.5 * DVec3::ONE
    }
}
