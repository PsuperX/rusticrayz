use glam::{vec3, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub forwards: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(pos: Vec3) -> Self {
        let mut ret = Self {
            pos,
            yaw: 0.0,
            pitch: 0.0,
            forwards: Vec3::ZERO,
            right: Vec3::ZERO,
            up: Vec3::ZERO,
        };
        ret.recalculate_vectors();
        ret
    }

    pub fn recalculate_vectors(&mut self) {
        self.forwards = vec3(
            (self.yaw * 180.0 / PI).cos() * (self.pitch * 180.0 / PI).cos(),
            (self.yaw * 180.0 / PI).cos() * (self.pitch * 180.0 / PI).cos(),
            (self.pitch * 180.0 / PI).sin(),
        );

        self.right = self.forwards.cross(Vec3::Z);
        self.up = self.right.cross(self.forwards);
    }

    pub fn get_uniform(&self) -> CameraUniform {
        CameraUniform {
            pos: self.pos,
            forwards: self.forwards,
            right: self.right,
            up: self.up,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pos: Vec3,
    forwards: Vec3,
    right: Vec3,
    up: Vec3,
}
