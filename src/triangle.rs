use glam::{vec3, Vec3};

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    corner_a: Vec3,
    _padding0: u32,
    corner_b: Vec3,
    _padding1: u32,
    corner_c: Vec3,
    _padding2: u32,

    normal_a: Vec3,
    _padding3: u32,
    normal_b: Vec3,
    _padding4: u32,
    normal_c: Vec3,
    _padding5: u32,

    color: Vec3,
    _padding6: u32,
}

impl Triangle {
    pub fn new(corners: [Vec3; 3], normals: [Vec3; 3], color: Vec3) -> Self {
        Self {
            corner_a: corners[0],
            corner_b: corners[1],
            corner_c: corners[2],
            normal_a: normals[0],
            normal_b: normals[1],
            normal_c: normals[2],
            color,
            ..Default::default()
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            corner_a: vec3(0.25, 0.25, -1.0),
            corner_b: vec3(-0.25, 0.25, -1.0),
            corner_c: vec3(0.0, -0.25, -1.0),
            normal_a: vec3(1.0, 0.0, 0.0),
            normal_b: vec3(0.0, 1.0, 0.0),
            normal_c: vec3(0.0, 0.0, 1.0),
            color: vec3(0.0, 0.5, 1.0),
            _padding0: Default::default(),
            _padding1: Default::default(),
            _padding2: Default::default(),
            _padding3: Default::default(),
            _padding4: Default::default(),
            _padding5: Default::default(),
            _padding6: Default::default(),
        }
    }
}
