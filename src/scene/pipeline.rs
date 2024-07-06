use iced::widget::shader::wgpu::{self, util::DeviceExt};
use iced::{Rectangle, Size};

mod buffer;
use buffer::Buffer;
mod vertex;
use vertex::Vertex;
use crate::texture;

pub mod particle;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    // particle vertex buffer
    vertices: wgpu::Buffer,
    // particles instance buffer
    particles: Buffer,
    // particle index buffer
    indices: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Self {
        //vertices of one cube
        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("particle vertex buffer"),
                contents: bytemuck::cast_slice(&particle::Raw::vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });

        //cube instance data
        let particles_buffer = Buffer::new(
            device,
            "particle instance buffer",
            std::mem::size_of::<particle::Raw>() as u64,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("particle index buffer"),
                contents: bytemuck::cast_slice(&particle::Raw::indices()),
                usage: wgpu::BufferUsages::INDEX,
            });

        let diffuse_bytes = include_bytes!("../../textures/particle-red.png");
        let diffuse_texture =
        texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "particle-red.png").unwrap();
        
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("particles uniform bind group layout"),
                entries: &[
                    // particle texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // texture sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let uniform_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("particles uniform bind group"),
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("particles pipeline layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("particles shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("./shaders/particles.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("particles pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), particle::Raw::desc()],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Max,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            pipeline,
            vertices,
            particles: particles_buffer,
            indices,
            uniform_bind_group
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        num_particles: usize,
        particles: &[particle::Raw],
    ) {
        //resize cubes vertex buffer if cubes amount changed
        let new_size = num_particles * std::mem::size_of::<particle::Raw>();
        self.particles.resize(device, new_size as u64);

        //always write new cube data since they are constantly rotating
        queue.write_buffer(&self.particles.raw, 0, bytemuck::cast_slice(particles));
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: Rectangle<u32>,
        num_particles: u32,
    ) {
        {
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("particles render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            pass.set_scissor_rect(
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height,
            );
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, self.particles.raw.slice(..));
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..num_particles);
        }
    }
}