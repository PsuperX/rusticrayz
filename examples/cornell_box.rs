use glam::{dvec3, DVec3};
use rusticrayz::{
    bvh::Bvh,
    camera::{Camera, CameraSettings},
    hittable::{Hittable, RotateY, Translate},
    material::*,
    shapes::{Quad, QuadBox},
};
use std::{io, sync::Arc};

fn main() -> io::Result<()> {
    let mut world: Vec<Box<dyn Hittable + Sync>> = vec![];

    let red = Arc::new(Lambertian::from_color(dvec3(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::from_color(dvec3(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(dvec3(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::from_color(dvec3(15.0, 15.0, 15.0)));

    world.push(Box::new(Quad::new(
        dvec3(555.0, 0.0, 0.0),
        dvec3(0.0, 555.0, 0.0),
        dvec3(0.0, 0.0, 555.0),
        green,
    )));
    world.push(Box::new(Quad::new(
        dvec3(0.0, 0.0, 0.0),
        dvec3(0.0, 555.0, 0.0),
        dvec3(0.0, 0.0, 555.0),
        red,
    )));
    world.push(Box::new(Quad::new(
        dvec3(343.0, 554.0, 332.0),
        dvec3(-130.0, 0.0, 0.0),
        dvec3(0.0, 0.0, -105.0),
        light,
    )));
    world.push(Box::new(Quad::new(
        dvec3(0.0, 0.0, 0.0),
        dvec3(555.0, 0.0, 0.0),
        dvec3(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.push(Box::new(Quad::new(
        dvec3(555.0, 555.0, 555.0),
        dvec3(-555.0, 0.0, 0.0),
        dvec3(0.0, 0.0, -555.0),
        white.clone(),
    )));
    world.push(Box::new(Quad::new(
        dvec3(0.0, 0.0, 555.0),
        dvec3(555.0, 0.0, 0.0),
        dvec3(0.0, 555.0, 0.0),
        white.clone(),
    )));

    let box1 = QuadBox::new(DVec3::ZERO, dvec3(165.0, 330.0, 165.0), white.clone());
    let box1 = RotateY::new(box1, 15.);
    let box1 = Translate::new(box1, dvec3(265., 0., 295.));
    world.push(Box::new(box1));

    let box2 = QuadBox::new(DVec3::ZERO, dvec3(165.0, 165.0, 165.0), white);
    let box2 = RotateY::new(box2, -18.);
    let box2 = Translate::new(box2, dvec3(130., 0., 65.));
    world.push(Box::new(box2));

    let world = Bvh::new(world);
    let camera = Camera::new(CameraSettings {
        image_width: 600,
        aspect_ratio: 1.,
        samples_per_pixel: 200,
        max_depth: 50,
        background: Some(DVec3::ZERO),
        look_from: Some(dvec3(278.0, 278.0, -800.0)),
        look_at: Some(dvec3(278.0, 278.0, 0.0)),
        view_up: Some(DVec3::Y),
        vfov: Some(40.),
        defocus_angle: Some(0.0),
        focus_dist: Some(10.),
    });
    camera.render_to_disk(&world)?;
    Ok(())
}
