struct Triangle {
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
    normal_a: vec3<f32>,
    normal_b: vec3<f32>,
    normal_c: vec3<f32>,
    color: vec3<f32>,
}

struct ObjectData {
    triangles: array<Triangle>,
}

struct Ray {
    dir: vec3<f32>,
    orig: vec3<f32>,
}

struct SceneData {
    camera_pos: vec3<f32>,
    camera_forwards: vec3<f32>,
    camera_right: vec3<f32>,
    camera_up: vec3<f32>,

    pixel00_loc: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,

    maxBounces: i32,
    samples_per_pixel: i32,
    primitiveCount: i32,
}

struct HitRecord {
    color: vec3<f32>,
    t: f32,
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    is_front_face: bool,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;
@group(0) @binding(2) var<storage, read> objects: ObjectData;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    // Initialize RNG
    seed = u32(screen_pos.y);
    seed = u32(screen_pos.x) + randi();
    randi();

    let pixel_center = scene.pixel00_loc + (f32(screen_pos.x) * scene.pixel_delta_u) + (f32(screen_pos.y) * scene.pixel_delta_v);
    let ray_direction = pixel_center - scene.camera_pos;

    var pixel_color: vec3<f32>;
    // for (var i = 0; i < scene.samples_per_pixel; i++) {
        // let ray = getRay(screen_pos.x, screen_pos.y, scene.pixel00_loc, scene.pixel_delta_u, scene.pixel_delta_v, scene.camera_pos);
        // pixel_color += rayColor(ray);
    // }
    // pixel_color /= f32(scene.samples_per_pixel);
    pixel_color = vec3<f32>(0.5, 0.5, 0.5);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn getRay(x: i32, y: i32, pixel00_loc: vec3<f32>, pixel_delta_u: vec3<f32>, pixel_delta_v: vec3<f32>, center: vec3<f32>) -> Ray {
    let pixel_center = pixel00_loc + (f32(x) * pixel_delta_u) + (f32(y) * pixel_delta_v);
    let pixel_sample = pixel_center + pixel_sample_square();

    let ray_origin = center;
    let ray_direction = pixel_sample - ray_origin;

    var ray: Ray;
    ray.orig = ray_origin;
    ray.dir = ray_direction;
    return ray;
}

fn pixel_sample_square() -> vec3<f32> {
    let px = -0.5 + rand();
    let py = -0.5 + rand();
    return (px * scene.pixel_delta_u) + (py * scene.pixel_delta_v);
}

fn rayColor(ray: Ray) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);

    var nearest_hit = 9999.0;
    var hit_something = false;

    var hit: HitRecord;

    for (var i = 0; i < scene.primitiveCount; i++) {
        var new_render_state: HitRecord = triangleIntersect(ray, objects.triangles[i], 0.001, nearest_hit, hit);

        if new_render_state.hit {
            nearest_hit = new_render_state.t;
            hit = new_render_state;
            hit_something = true;
        }
    }

    if hit_something {
        return hit.normal;
    }

    // Miss
    let unit_dir = normalize(ray.dir);
    let a = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}

fn triangleIntersect(ray: Ray, triangle: Triangle, t_min: f32, t_max: f32, oldHit: HitRecord) -> HitRecord {
    var hit: HitRecord;
    hit.hit = false;

    let e1 = triangle.corner_b - triangle.corner_a;
    let e2 = triangle.corner_c - triangle.corner_a;
    let h = cross(ray.dir, e2);
    let a = dot(e1, h);

    if abs(a) < 0.00001 {
        return hit; // The ray is nearly parallel to the triangle
    }

    let f = 1.0 / a;
    let s = ray.orig - triangle.corner_a;
    let u = f * dot(s, h);

    if u < 0.0 || u > 1.0 {
        return hit; // The intersection point is outside the triangle
    }

    let q = cross(s, e1);
    let v = f * dot(ray.dir, q);

    if v < 0.0 || u + v > 1.0 {
        return hit; // The intersection point is outside the triangle
    }

    let t = f * dot(e2, q);

    if t > 0.00001 {
        hit.hit = true;
        hit.color = triangle.color;
        hit.t = t;
        // TODO: We might end up hitting something closer as we do our search,
        // and we will only need the normal of the closest thing
        hit.position = ray.orig + t * ray.dir;
        hit.normal = normalize((1.0 - u - v) * triangle.normal_a + u * triangle.normal_b + v * triangle.normal_c);
        hit.is_front_face = dot(ray.dir, hit.normal) < 0.0;
        return hit;
    }

    return hit;
}


// Random number generator
var<private> seed: u32;

// --- choose one:
// Returns a random integer
// fn randi(x: u32) {
//     seed = lowbias32(x);
//     return seed;
// }

// Returns a random integer
fn randi() -> u32 {
    seed = triple32(seed);
    return seed;
}

// Returns a random real in [0,1).
fn rand() -> f32 {
    return f32(randi()) / f32(0xffffffffu);
}

// Returns a random real in [min,max).
fn rand_range(min: f32, max: f32) -> f32 {
    return min + (max - min) * rand();
}

// Source: https://www.shadertoy.com/view/WttXWX
//bias: 0.17353355999581582 ( very probably the best of its kind )
fn lowbias32(seed: u32) -> u32 {
    var x = seed;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    x ^= x >> 16u;
    return x;
}

// bias: 0.020888578919738908 = minimal theoretic limit
fn triple32(seed: u32) -> u32 {
    var x = seed;
    x ^= x >> 17u;
    x *= 0xed5ad4bbu;
    x ^= x >> 11u;
    x *= 0xac4c1b51u;
    x ^= x >> 15u;
    x *= 0x31848babu;
    x ^= x >> 14u;
    return x;
}
