use cgmath::{Matrix4, Rad, SquareMatrix}; // Added Rad import
use bytemuck::{Pod, Zeroable};
use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            model: Matrix4::identity().into(),
        }
    }

    pub fn update_model(&mut self) {
        // Set the model matrix to identity to prevent any rotation
        self.model = Matrix4::identity().into();
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        let view = Matrix4::look_at_rh(camera.eye, camera.target, camera.up);
        let projection = cgmath::perspective(Rad(camera.fovy), camera.aspect, camera.znear, camera.zfar);
        self.view_proj = (projection * view).into();
    }
}