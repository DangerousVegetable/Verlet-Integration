use std::cmp::Ordering;
use std::iter;

use glam::Vec2;
use iced::mouse;
use iced::time::Duration;
use iced::widget::shader::{self, wgpu};
use iced::Rectangle;
use rand::Rng;

mod pipeline;
use pipeline::particle::{self};
use pipeline::Pipeline;
use pipeline::Uniforms;

mod camera;
pub use camera::Camera;

use crate::particle::Particle;

pub const MAX: u32 = 1000000;

#[derive(Clone)]
pub struct Scene {
    pub particles: Vec<Particle>,
    pub camera: Camera,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self { particles: vec![], camera: Camera::default()};

        scene.change_number(10000);

        scene
    }

    pub fn change_number(&mut self, number: u32) {
        let curr_particles = self.particles.len() as u32;

        match number.cmp(&curr_particles) {
            Ordering::Greater => {
                // spawn
                let particles_2_spawn = (number - curr_particles) as usize;

                let mut particles = 0;
                self.particles.extend(iter::from_fn(|| {
                    if particles < particles_2_spawn {
                        particles += 1;
                        Some(Particle::new(rand::thread_rng().gen_range(0.1..0.5), rnd_origin()))
                    } else {
                        None
                    }
                }));
            }
            Ordering::Less => {
                // chop
                let particles_2_cut = curr_particles - number;
                let new_len = self.particles.len() - particles_2_cut as usize;
                self.particles.truncate(new_len);
            }
            Ordering::Equal => {}
        }
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(
            &self.particles,
            &self.camera,
            bounds,
        )
    }
}

/// A collection of `Particles`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
    particles: Vec<particle::Raw>,
    uniforms: Uniforms,
}

impl Primitive {
    pub fn new(
        particles: &[Particle],
        camera: &Camera,
        bounds: Rectangle,
    ) -> Self {
        Self {
            particles: particles
                .iter()
                .map(particle::Raw::from_particle)
                .collect::<Vec<particle::Raw>>(),
            uniforms: Uniforms::new(camera, bounds)
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: Rectangle,
        target_size: iced::Size<u32>,
        _scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                target_size,
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Upload data to GPU
        pipeline.update(
            device,
            queue,
            target_size,
            &self.uniforms,
            self.particles.len(),
            &self.particles,
        );
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        target_size: iced::Size<u32>,
        viewport: Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Render primitive
        pipeline.render(
            target,
            encoder,
            viewport,
            self.particles.len() as u32
        );
    }
}

fn rnd_origin() -> Vec2 {
    Vec2::new(
        rand::thread_rng().gen_range(-25.0..25.0),
        rand::thread_rng().gen_range(-25.0..25.0),
    )
}
