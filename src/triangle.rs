use glam::Vec3;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    corner_a: Vec3,
    _padding0: u32,
    corner_b: Vec3,
    _padding1: u32,
    corner_c: Vec3,
    _padding2: u32,
    color: Vec3,
    _padding3: u32,
}

impl Triangle {
    pub fn new(corners: [Vec3; 3], color: Vec3) -> Self {
        Self {
            corner_a: corners[0],
            corner_b: corners[1],
            corner_c: corners[2],
            color,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}
