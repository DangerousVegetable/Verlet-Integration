use iced::widget::shader::wgpu;

use glam::vec2;

use super::vertex::Vertex;
use crate::particle::Particle;

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct Raw {
    size: f32,
    pos: glam::Vec2,
}

impl Raw {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        // size
        2 => Float32,
        // position
        3 => Float32x2,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Raw {
    pub fn from_particle(particle: &Particle) -> Raw {
        Raw {
            size: particle.radius,
            pos: particle.pos,
        }
    }

    pub fn vertices() -> [Vertex; 4] {
        [
            Vertex {
                pos: vec2(-1.0, 1.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec2(-1.0, -1.0),
                uv: vec2(0.0, 1.0)
            },
            Vertex {
                pos: vec2(1.0, -1.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec2(1.0, 1.0),
                uv: vec2(1.0, 0.0),
            },
        ]
    }

    pub fn indices() -> [u16; 6] {
        // two faces: 0-1-3 and 3-1-2
        [0,1,3,3,1,2]
    }
}