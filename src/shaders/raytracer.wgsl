#import bevy_render::view::View

struct Ray {
    dir: vec3<f32>,
    inv_dir: vec3<f32>,
    orig: vec3<f32>,
}

struct Aabb {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct HitRecord {
    color: vec3<f32>,
    t: f32,
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    is_front_face: bool,
}

struct Intersection {
    uv: vec2<f32>,
    distance: f32,
}

struct Hit {
    intersection: Intersection,
    instance_index: u32,
    primitive_index: u32,
}

struct Vertex {
    position: vec3<f32>,
    u: f32,
    normal: vec3<f32>,
    v: f32,
}

struct PrimitiveVertex {
    position: vec3<f32>,
    index: u32,
}

struct Primitive {
    vertices: array<PrimitiveVertex, 3>,
}

struct MeshIndex {
    vertex: u32,
    primitive: u32,
    node: vec2<u32>,    // x: offset, y: size
}

struct Instance {
    min: vec3<f32>,
    material: u32,
    max: vec3<f32>,
    node_index: u32,
    model: mat4x4<f32>,
    inverse_transpose_model: mat4x4<f32>,
    mesh: MeshIndex,
}

struct Node {
    min: vec3<f32>,
    entry_index: u32,
    max: vec3<f32>,
    exit_index: u32,
}

struct Nodes {
    count: u32,
    data: array<Node>,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0) var<storage, read> vertex_buffer: array<Vertex>;
@group(1) @binding(1) var<storage, read> primitive_buffer: array<Primitive>;
@group(1) @binding(2) var<storage, read> primitive_node_buffer: Nodes;
@group(1) @binding(3) var<storage, read> instance_buffer: array<Instance>;
@group(1) @binding(4) var<storage, read> instance_node_buffer: Nodes;

@group(2) @binding(0) var<uniform> view: View;

const F32_MAX: f32 = 3.4028235e38;
const U32_MAX: u32 = 0xFFFFFFFFu;
const BVH_LEAF_FLAG: u32 = 0x80000000u;

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

    var pixel_color: vec3<f32>;
    // TODO: this
    let samples_per_pixel = 1;
    for (var i = 0; i < samples_per_pixel; i++) {
        let ray = get_ray(screen_pos, screen_size);
        pixel_color += ray_color(ray);
    }
    pixel_color /= f32(samples_per_pixel);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn get_ray(screen_pos: vec2<i32>, screen_size: vec2<i32>) -> Ray {
    let pixelCenter = vec2<f32>(screen_pos) + vec2<f32>(0.5);
    let inUV = pixelCenter / vec2<f32>(screen_size);
    let d = inUV * 2.0 - 1.0;

    let origin = view.view * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let pixel_center = view.inverse_projection * vec4<f32>(d.x, -d.y, 1.0, 1.0);
    // TODO: use pixel_sample_square
    let direction = view.view * vec4<f32>(normalize(pixel_center.xyz), 0.0);

    var ray: Ray;
    ray.orig = vec3<f32>(origin.xyz);
    ray.dir = vec3<f32>(direction.xyz);
    ray.inv_dir = 1.0 / vec3<f32>(direction.xyz);
    return ray;
}

fn pixel_sample_square() -> vec3<f32> {
    let px = -0.5 + rand();
    let py = -0.5 + rand();
    // return (px * scene.pixel_delta_u) + (py * scene.pixel_delta_v);
    return vec3<f32>(0.0, 0.0, 0.0);
}

fn ray_color(ray: Ray) -> vec3<f32> {
    var new_render_state = traverse_instances(ray, 0.0, F32_MAX);
    if new_render_state.instance_index != U32_MAX {
        return vec3<f32>(new_render_state.intersection.uv, 0.0);
    }

    // Miss
    let unit_dir = normalize(ray.dir);
    let a = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - a) * vec3<f32>(1.0, 1.0, 1.0) + a * vec3<f32>(0.5, 0.7, 1.0);
}

fn traverse_instances(ray: Ray, early_distance: f32, max_distance: f32) -> Hit {
    var hit: Hit;
    hit.intersection.distance = max_distance;
    hit.instance_index = U32_MAX;
    hit.primitive_index = U32_MAX;

    var index = 0u;
    for (; index < instance_node_buffer.count;) {
        let node = instance_node_buffer.data[index];
        var aabb: Aabb;

        if node.entry_index >= BVH_LEAF_FLAG {
            let instance_index = node.entry_index - BVH_LEAF_FLAG;
            let instance = instance_buffer[instance_index];
            aabb.min = instance.min;
            aabb.max = instance.max;

            if intersects_aabb(ray, aabb) < hit.intersection.distance {
                var r: Ray;
                r.orig = instance_position_world_to_local(instance, ray.orig);
                r.dir = instance_direction_world_to_local(instance, ray.dir);
                r.inv_dir = 1.0 / r.dir;

                if traverse_mesh(&hit, r, instance.mesh, early_distance) {
                    hit.instance_index = instance_index;
                    if hit.intersection.distance < early_distance {
                        return hit;
                    }
                }
            }

            index = node.exit_index;
        } else {
            aabb.min = node.min;
            aabb.max = node.max;
            index = select(
                node.exit_index,
                node.entry_index,
                intersects_aabb(ray, aabb) < hit.intersection.distance
            );
        }
    }

    return hit;
}

fn traverse_mesh(hit: ptr<function, Hit>, ray: Ray, mesh: MeshIndex, early_distance: f32) -> bool {
    var intersected = false;
    var index = 0u;
    for (; index < mesh.node.y;) {
        let node_index = mesh.node.x + index;
        let node = primitive_node_buffer.data[node_index];
        var aabb: Aabb;
        if node.entry_index >= BVH_LEAF_FLAG {
            let primitive_index = mesh.primitive + node.entry_index - BVH_LEAF_FLAG;
            let vertices = primitive_buffer[primitive_index].vertices;

            aabb.min = min(vertices[0].position, min(vertices[1].position, vertices[2].position));
            aabb.max = max(vertices[0].position, max(vertices[1].position, vertices[2].position));

            if intersects_aabb(ray, aabb) < (*hit).intersection.distance {
                let intersection = intersects_triangle(ray, vertices);
                if intersection.distance < (*hit).intersection.distance {
                    (*hit).intersection = intersection;
                    (*hit).primitive_index = primitive_index;
                    intersected = true;

                    if intersection.distance < early_distance {
                        return intersected;
                    }
                }
            }

            index = node.exit_index;
        } else {
            aabb.min = node.min;
            aabb.max = node.max;
            index = select(
                node.exit_index,
                node.entry_index,
                intersects_aabb(ray, aabb) < (*hit).intersection.distance
            );
        }
    }

    return intersected;
}

fn instance_position_world_to_local(instance: Instance, p: vec3<f32>) -> vec3<f32> {
    let inverse_model = transpose(instance.inverse_transpose_model);
    let position = inverse_model * vec4<f32>(p, 1.0);
    return position.xyz / position.w;
}

fn instance_direction_world_to_local(instance: Instance, p: vec3<f32>) -> vec3<f32> {
    let inverse_model = transpose(instance.inverse_transpose_model);
    let direction = inverse_model * vec4<f32>(p, 0.0);
    return direction.xyz;
}

fn intersects_aabb(ray: Ray, aabb: Aabb) -> f32 {
    let t1 = (aabb.min - ray.orig) * ray.inv_dir;
    let t2 = (aabb.max - ray.orig) * ray.inv_dir;

    var t_min = min(t1.x, t2.x);
    var t_max = max(t1.x, t2.x);

    t_min = max(t_min, min(t1.y, t2.y));
    t_max = min(t_max, max(t1.y, t2.y));

    t_min = max(t_min, min(t1.z, t2.z));
    t_max = min(t_max, max(t1.z, t2.z));

    var t: f32 = F32_MAX;
    if t_max >= t_min && t_max >= 0.0 {
        t = t_min;
    }
    return t;
}

fn intersects_triangle(ray: Ray, triangle: array<PrimitiveVertex, 3>) -> Intersection {
    var hit: Intersection;
    hit.distance = F32_MAX;

    let e1 = triangle[1].position - triangle[0].position;
    let e2 = triangle[2].position - triangle[0].position;
    let h = cross(ray.dir, e2);
    let a = dot(e1, h);

    if abs(a) < 0.00001 {
        return hit; // The ray is nearly parallel to the triangle
    }

    let f = 1.0 / a;
    let s = ray.orig - triangle[0].position;
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

    hit.distance = t;
    hit.uv = vec2<f32>(u, v);
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
