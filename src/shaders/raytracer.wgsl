struct Ray {
    dir: vec3<f32>,
    orig: vec3<f32>,
}

struct SceneData {
    cameraPos: vec3<f32>,
    cameraForwards: vec3<f32>,
    cameraRight: vec3<f32>,
    cameraUp: vec3<f32>,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    // TODO: This probably should happen outside the shader
    let focal_length: f32 = 1.0;
    let viewport_height: f32 = 2.0;
    let viewport_width: f32 = viewport_height * (f32(screen_size.x) / f32(screen_size.y));
    let camera_center: vec3<f32> = scene.cameraPos;

    // Calculate the vectors across the horizontal and down the vertical viewport edges.
    let viewport_u: vec3<f32> = vec3<f32>(viewport_width, 0.0, 0.0);
    let viewport_v: vec3<f32> = vec3<f32>(0.0, -viewport_height, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel.
    let pixel_delta_u: vec3<f32> = viewport_u / f32(screen_size.x);
    let pixel_delta_v: vec3<f32> = viewport_v / f32(screen_size.y);

    // Calculate the location of the upper left pixel.
    let viewport_upper_left: vec3<f32> = camera_center - vec3<f32>(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc: vec3<f32> = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);


    let pixel_center: vec3<f32> = pixel00_loc + (f32(screen_pos.x) * pixel_delta_u) + (f32(screen_pos.y) * pixel_delta_v);
    let ray_direction: vec3<f32> = pixel_center - camera_center;

    var ray: Ray;
    ray.dir = ray_direction;
    ray.orig = camera_center;

    let pixel_color: vec3<f32> = rayColor(ray);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn rayColor(ray: Ray) -> vec3<f32> {
    let unit_dir: vec3<f32> = normalize(ray.dir);
    let a = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}
