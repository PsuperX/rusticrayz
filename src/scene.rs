use crate::camera::Camera;

pub struct Scene {
    camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }

    pub fn wih_primitives(camera: Camera) -> Self {
        Self { camera }
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }
}
