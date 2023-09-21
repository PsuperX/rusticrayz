use glam::Vec3;
use std::io::Write;

pub type Color = Vec3;

pub fn write_color(out: &mut dyn Write, pixel_color: &Color) {
    write!(
        out,
        "{} {} {} ",
        255.99 * pixel_color.x,
        255.99 * pixel_color.y,
        255.99 * pixel_color.z
    )
    .unwrap();
}
