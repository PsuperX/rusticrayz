use glam::{dvec3, DVec3};
use rusticrayz::{
    bvh::Bvh,
    camera::{Camera, CameraSettings},
    material::*,
    shapes::Quad,
    texture::*,
};
use std::{io, sync::Arc};

fn main() -> io::Result<()> {
    let mut world = vec![];

    let left_red = Arc::new(Lambertian::new(SolidColor::from(dvec3(1.0, 0.2, 0.2))));
    let back_green = Arc::new(Lambertian::new(SolidColor::from(dvec3(0.2, 1.0, 0.2))));
    let right_blue = Arc::new(Lambertian::new(SolidColor::from(dvec3(0.2, 0.2, 1.0))));
    let upper_orange = Arc::new(Lambertian::new(SolidColor::from(dvec3(1.0, 0.5, 0.0))));
    let lower_teal = Arc::new(Lambertian::new(SolidColor::from(dvec3(0.2, 0.8, 0.8))));

    world.push(Quad::new(
        dvec3(-3.0, -2.0, 5.0),
        dvec3(0.0, 0.0, -4.0),
        dvec3(0.0, 4.0, 0.0),
        left_red,
    ));
    world.push(Quad::new(
        dvec3(-2.0, -2.0, 0.0),
        dvec3(4.0, 0.0, 0.0),
        dvec3(0.0, 4.0, 0.0),
        back_green,
    ));
    world.push(Quad::new(
        dvec3(3.0, -2.0, 1.0),
        dvec3(0.0, 0.0, 4.0),
        dvec3(0.0, 4.0, 0.0),
        right_blue,
    ));
    world.push(Quad::new(
        dvec3(-2.0, 3.0, 1.0),
        dvec3(4.0, 0.0, 0.0),
        dvec3(0.0, 0.0, 4.0),
        upper_orange,
    ));
    world.push(Quad::new(
        dvec3(-2.0, -3.0, 5.0),
        dvec3(4.0, 0.0, 0.0),
        dvec3(0.0, 0.0, -4.0),
        lower_teal,
    ));

    let world = Bvh::new(world);
    let camera = Camera::new(CameraSettings {
        image_width: 400,
        aspect_ratio: 1.,
        samples_per_pixel: 100,
        max_depth: 50,
        look_from: Some(dvec3(0., 0., 9.)),
        look_at: Some(DVec3::ZERO),
        view_up: Some(DVec3::Y),
        vfov: Some(80.),
        defocus_angle: Some(0.0),
        focus_dist: Some(10.),
    });
    camera.render_to_disk(&world)?;
    Ok(())
}
