struct Triangle {
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
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
    primitiveCount: i32,
}

struct HitRecord {
    color: vec3<f32>,
    t: f32,
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;
@group(0) @binding(2) var<storage, read> objects: ObjectData;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    let pixel_center: vec3<f32> = scene.pixel00_loc + (f32(screen_pos.x) * scene.pixel_delta_u) + (f32(screen_pos.y) * scene.pixel_delta_v);
    let ray_direction: vec3<f32> = pixel_center - scene.camera_pos;

    var ray: Ray;
    ray.dir = ray_direction;
    ray.orig = scene.camera_pos;

    let pixel_color: vec3<f32> = rayColor(ray);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn rayColor(ray: Ray) -> vec3<f32> {
    var color: vec3<f32> = vec3(0.0, 0.0, 0.0);

    var nearest_hit: f32 = 9999.0;
    var hit_something: bool = false;

    var hit: HitRecord;

    for (var i: i32 = 0; i < scene.primitiveCount; i++) {
        var new_render_state: HitRecord = triangleIntersect(ray, objects.triangles[i], 0.001, nearest_hit, hit);

        if new_render_state.hit {
            nearest_hit = new_render_state.t;
            hit = new_render_state;
            hit_something = true;
        }
    }

    if hit_something {
        return hit.color;
        // return hit.position;
    }

    // Miss
    let unit_dir: vec3<f32> = normalize(ray.dir);
    let a = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}

fn triangleIntersect(ray: Ray, triangle: Triangle, t_min: f32, t_max: f32, oldHit: HitRecord) -> HitRecord {
    // Set up a blank hitRecord,
    // right now this hasn't hit anything
    var result: HitRecord;
    result.color = oldHit.color;

    // Compute the triangle's normal and a vector on the triangle's plane
    var N: vec3<f32> = cross(triangle.corner_b - triangle.corner_a, triangle.corner_c - triangle.corner_a);
    var d: f32 = -dot(N, triangle.corner_a);

    // Compute the ray-plane intersection point
    var t: f32 = -(dot(N, ray.orig) + d) / dot(N, ray.dir);

    // Check if the intersection point is in range
    if t < t_min && t > t_max {
        return result;
    }

    // Calculate the intersection point
    var intersection_point: vec3<f32> = ray.orig + t * ray.dir;

    // Calculate vectors between the intersection point and triangle vertices
    var e0: vec3<f32> = triangle.corner_b - triangle.corner_a;
    var e1: vec3<f32> = triangle.corner_c - triangle.corner_b;
    var e2: vec3<f32> = triangle.corner_a - triangle.corner_c;
    var c0: vec3<f32> = intersection_point - triangle.corner_a;
    var c1: vec3<f32> = intersection_point - triangle.corner_b;
    var c2: vec3<f32> = intersection_point - triangle.corner_c;

    var h0: vec3<f32> = cross(e0, c0);
    var h1: vec3<f32> = cross(e1, c1);
    var h2: vec3<f32> = cross(e2, c2);

    // Perform the edge tests
    if dot(N, h0) >= 0.0 && dot(N, h1) >= 0.0 && dot(N, h2) >= 0.0 {
        // Intersection occurred
        result.t = t;
        result.color = triangle.color;
        result.hit = true;
        result.position = intersection_point;
        result.normal = N;
        return result;
    }

    result.hit = false;
    return result;
}
