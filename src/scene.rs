use crate::{camera::Camera, triangle::Triangle};

pub struct Scene {
    primitives: Vec<Triangle>,
    camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            primitives: vec![],
            camera,
        }
    }

    pub fn wih_primitives(primitives: Vec<Triangle>, camera: Camera) -> Self {
        Self { primitives, camera }
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_primitives(&self) -> &[Triangle] {
        &self.primitives
    }
}
