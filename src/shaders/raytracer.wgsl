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

struct HitInfo {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    instance_index: u32,
    material_index: u32,
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

struct Material {
    base_color: vec4<f32>,
    base_color_texture: u32,
    emissive: vec4<f32>,
    emissive_texture: u32,
    perceptual_roughness: f32,
    metallic: f32,
    metallic_roughness_texture: u32,
    reflectance: f32,
    normal_map_texture: u32,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;

@group(1) @binding(0) var<storage, read> vertex_buffer: array<Vertex>;
@group(1) @binding(1) var<storage, read> primitive_buffer: array<Primitive>;
@group(1) @binding(2) var<storage, read> primitive_node_buffer: Nodes;
@group(1) @binding(3) var<storage, read> material_buffer: array<Material>;
@group(1) @binding(4) var<storage, read> instance_buffer: array<Instance>;
@group(1) @binding(5) var<storage, read> instance_node_buffer: Nodes;

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var samplers: binding_array<sampler>;

@group(3) @binding(0) var<uniform> view: View;

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

    var pixel_color: vec4<f32>;
    // TODO: this
    let samples_per_pixel = 1;
    for (var i = 0; i < samples_per_pixel; i++) {
        pixel_color += per_pixel(screen_pos, screen_size);
    }
    pixel_color /= f32(samples_per_pixel);

    textureStore(color_buffer, screen_pos, pixel_color);
}

fn per_pixel(screen_pos: vec2<i32>, screen_size: vec2<i32>) -> vec4<f32> {
    var ray = get_ray(screen_pos, screen_size);

    var light = vec4<f32>(0.0);
    var contribution = vec4<f32>(1.0);

    // TODO: this
    let MAX_BOUNCES = 5;
    for (var i = 0; i < MAX_BOUNCES; i++) {
        let hit = trace_ray(ray);
        if hit.instance_index == U32_MAX {
            // Miss
            let unit_dir = normalize(ray.dir);
            let a = 0.5 * (unit_dir.y + 1.0);
            let sky_color = (1.0 - a) * vec4<f32>(1.0) + a * vec4<f32>(0.5, 0.7, 1.0, 1.0);

            // light += sky_color * contribution;
            break;
        }

        let material = material_buffer[hit.material_index];

        // Albedo
        var albedo = material.base_color;
        let albedo_idx = material.base_color_texture;
        if albedo_idx != U32_MAX {
            albedo *= textureSampleLevel(textures[albedo_idx], samplers[albedo_idx], hit.uv, 0.0);
        }
        contribution *= albedo;

        // Emissive
        var emissive = material.emissive;
        let emissive_idx = material.emissive_texture;
        if emissive_idx != U32_MAX {
            emissive *= textureSampleLevel(textures[emissive_idx], samplers[emissive_idx], hit.uv, 0.0);
        }
        light += emissive * contribution;

        ray.orig = hit.position + hit.normal * 0.0001;
        ray.dir = normalize(hit.normal + rand_unit());
        ray.inv_dir = 1.0 / ray.dir;
    }

    return light;
}

fn trace_ray(ray: Ray) -> HitInfo {
    let new_render_state = traverse_instances(ray, 0.0, F32_MAX);
    if new_render_state.instance_index != U32_MAX {
        return closest_hit(ray, new_render_state);
    }
    return miss(ray);
}

fn closest_hit(ray: Ray, hit: Hit) -> HitInfo {
    var info: HitInfo;
    info.instance_index = hit.instance_index;

    let instance = instance_buffer[hit.instance_index];
    let primitive = primitive_buffer[hit.primitive_index].vertices;

    let vertex0 = vertex_buffer[instance.mesh.vertex + primitive[0].index];
    let vertex1 = vertex_buffer[instance.mesh.vertex + primitive[1].index];
    let vertex2 = vertex_buffer[instance.mesh.vertex + primitive[2].index];

    let uv0 = vec2<f32>(vertex0.u, vertex0.v);
    let uv1 = vec2<f32>(vertex1.u, vertex1.v);
    let uv2 = vec2<f32>(vertex2.u, vertex2.v);

    let uv = hit.intersection.uv;
    info.uv = uv.x * uv1 + uv.y * uv2 + (1.0 - uv.x - uv.y) * uv0;
    let normal = uv.x * vertex1.normal + uv.y * vertex2.normal + (1.0 - uv.x - uv.y) * vertex0.normal;
    info.normal = instance_direction_local_to_world(instance, normal);

    info.position = ray.orig + ray.dir * hit.intersection.distance;
    info.material_index = instance.material;

    return info;
}

fn miss(ray: Ray) -> HitInfo {
    var info: HitInfo;
    info.instance_index = U32_MAX;
    info.material_index = U32_MAX;
    return info;
}

fn get_ray(screen_pos: vec2<i32>, screen_size: vec2<i32>) -> Ray {
    let pixelCenter = vec2<f32>(screen_pos) + vec2<f32>(rand() - 0.5, rand() - 0.5);
    let inUV = pixelCenter / vec2<f32>(screen_size);
    let d = inUV * 2.0 - 1.0;

    let origin = view.view * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let pixel_center = view.inverse_projection * vec4<f32>(d.x, -d.y, 1.0, 1.0);
    let direction = view.view * vec4<f32>(normalize(pixel_center.xyz), 0.0);

    var ray: Ray;
    ray.orig = vec3<f32>(origin.xyz);
    ray.dir = vec3<f32>(direction.xyz);
    ray.inv_dir = 1.0 / vec3<f32>(direction.xyz);
    return ray;
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

fn instance_direction_local_to_world(instance: Instance, p: vec3<f32>) -> vec3<f32> {
    let direction = instance.model * vec4<f32>(p, 0.0);
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

#ifdef CULLING
    if a < 0.00001 {
        return hit; // The ray is nearly parallel to the triangle
    }
#else
    if abs(a) < 0.00001 {
        return hit; // The ray is nearly parallel to the triangle
    }
#endif

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

// Return a random vec3 in the range [-1, 1]
fn rand_vec3() -> vec3<f32> {
    return vec3<f32>(rand(), rand(), rand()) * 2.0 - vec3<f32>(1.0);
}

// Returns a random unit vector
fn rand_unit() -> vec3<f32> {
    return normalize(tan(rand_vec3()));
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
