mod color;
mod ray;

use color::{write_color, Color};
use glam::{vec3, Vec3};
use ray::Ray;

fn main() {
    let aspect_ratio = 16f32 / 9f32;
    let image_width = 400;

    let image_height = (image_width as f32 / aspect_ratio) as i32;
    let image_height = if image_height < 1 { 1 } else { image_height };

    let focal_lenght = 1f32;
    let viewport_height = 2f32;
    let viewport_width = viewport_height * (image_width as f32 / image_height as f32);
    let camera_center = Vec3::ZERO;

    let viewport_u = vec3(viewport_width, 0., 0.);
    let viewport_v = vec3(0., -viewport_height, 0.);

    let pixel_delta_u = viewport_u / image_width as f32;
    let pixel_delta_v = viewport_v / image_height as f32;

    let viewport_upper_left =
        camera_center - vec3(0., 0., focal_lenght) - viewport_u / 2. - viewport_v / 2.;

    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    println!("P3 {} {} 255", image_width, image_height);

    for j in 0..image_height {
        eprintln!("\rScanlines remaining: {} ", image_height - j);
        for i in 0..image_width {
            let pixel_center =
                pixel00_loc + (i as f32 * pixel_delta_u) + (j as f32 * pixel_delta_v);
            let ray_dir = pixel_center - camera_center;

            let pixel_color = ray_color(&Ray::new(camera_center, ray_dir));
            write_color(&mut std::io::stdout(), &pixel_color);
        }
    }

    eprintln!("\rDone.");
}

fn ray_color(r: &Ray) -> Color {
    let unit_dir = r.dir.normalize();
    let a = 0.5 * (unit_dir.y + 1.);
    Color::lerp(Color::ONE, Color::new(0.5, 0.7, 1.), a)
}
