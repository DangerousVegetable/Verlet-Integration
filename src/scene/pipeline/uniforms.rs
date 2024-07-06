use crate::scene::Camera;

use iced::Rectangle;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    camera_proj: glam::Mat4,
}

impl Uniforms {
    pub fn new(camera: &Camera, bounds: Rectangle) -> Self {
        let camera_proj = camera.proj_matrix(bounds);

        Self {
            camera_proj
        }
    }
}