use glam::{vec2, Mat4, Vec2, vec4};
use iced::Rectangle;

#[derive(Clone, Copy)]
pub struct Camera {
    pub pos: Vec2,
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: vec2(0., 0.),
            fov: 50.,
        }
    }
}

impl Camera {
    pub fn proj_matrix(&self, bounds: Rectangle) -> glam::Mat4 {
        let scale_x = 2./self.fov;
        let scale_y = scale_x * bounds.width/bounds.height;
        Mat4::from_cols(
            vec4(scale_x, 0., 0., 0.),
            vec4(0., scale_y, 0., 0.),
            vec4(0., 0., 1., 0.),
            vec4(-self.pos.x*scale_x, -self.pos.y*scale_y, 0., 1.),
        )
    }
}
