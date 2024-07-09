use std::cmp::Ordering;
use std::iter;

use glam::{Vec2, vec2};
use iced::mouse;
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
use crate::solver::{self, Simulation};

#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub simulation: Simulation,
}

impl Scene {
    pub fn new(number: u32, constraint: solver::Constraint) -> Self {
        let mut scene = Self { 
            camera: Camera::default(), 
            simulation: Simulation::new(constraint, 2.*solver::PARTICLE_SIZE, &[])};

        scene.change_number(number);

        scene
    }

    pub fn update(&mut self, dt: f32) {
        self.simulation.solve(dt);
    }

    pub fn change_number(&mut self, number: u32) {
        self.simulation.change_number(number)
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
            &self.simulation.particles,
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
        _target_size: iced::Size<u32>,
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


