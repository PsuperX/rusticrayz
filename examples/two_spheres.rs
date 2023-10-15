use glam::{dvec3, DVec3};
use rusticrayz::{
    bvh::Bvh,
    camera::{Camera, CameraSettings},
    material::*,
    shapes::Sphere,
    texture::*,
};
use std::{io, sync::Arc};

fn main() -> io::Result<()> {
    let mut world = vec![];

    let checker: CheckerTexture<SolidColor, SolidColor> =
        CheckerTexture::new(0.32, dvec3(0.2, 0.3, 0.1).into(), DVec3::splat(0.9).into());
    let checker_material = Arc::new(Lambertian::new(checker));
    world.push(Sphere::new(
        dvec3(0., -10., 0.),
        10.,
        checker_material.clone(),
    ));
    world.push(Sphere::new(dvec3(0., 10., 0.), 10., checker_material));

    let world = Bvh::new(world);
    let camera = Camera::new(CameraSettings {
        image_width: 400,
        aspect_ratio: 16. / 9.,
        samples_per_pixel: 100,
        max_depth: 50,
        look_from: Some(dvec3(13., 2., 3.)),
        look_at: Some(DVec3::ZERO),
        view_up: Some(DVec3::Y),
        vfov: Some(20.),
        defocus_angle: Some(0.0),
        focus_dist: Some(10.),
    });
    camera.render_to_disk(&world)?;
    Ok(())
}
