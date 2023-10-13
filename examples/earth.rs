use glam::{dvec3, DVec3};
use rusticrayz::{
    bvh::Bvh,
    camera::{Camera, CameraSettings},
    material::*,
    sphere::Sphere,
    texture::*,
};
use std::{io, sync::Arc};

fn main() -> io::Result<()> {
    let mut world = vec![];

    let earth_texture = ImageTexture::load_image("assets/earth.jpg").unwrap();
    let earth_surface = Arc::new(Lambertian::new(earth_texture));
    world.push(Sphere::new(dvec3(0., 0., 0.), 2., earth_surface));

    let world = Bvh::new(world);
    let camera = Camera::new(CameraSettings {
        image_width: 400,
        aspect_ratio: 16. / 9.,
        samples_per_pixel: 100,
        max_depth: 50,
        look_from: Some(dvec3(0., 0., 12.)),
        look_at: Some(DVec3::ZERO),
        view_up: Some(DVec3::Y),
        vfov: Some(20.),
        defocus_angle: Some(0.0),
        focus_dist: Some(10.),
    });
    camera.render_to_disk(&world)?;
    Ok(())
}
