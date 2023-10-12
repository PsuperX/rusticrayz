use crate::color::Color;
use glam::DVec3;

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

impl From<Color> for SolidColor {
    fn from(value: Color) -> Self {
        Self { color: value }
    }
}
