// Vertex shader

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct ParticleInput {
    @location(2) size: f32, 
    @location(3) position: vec2<f32>,
    @location(4) texture: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texture: u32,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    particle: ParticleInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;
    out.texture = particle.texture;
    let world_position = vec4<f32>(vertex.position*particle.size + particle.position, 0.0, 1.0);
    out.clip_position = uniforms.projection * world_position;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var texture_array: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var sampler_array: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(
        texture_array[in.texture], 
        sampler_array[in.texture], 
        in.uv);
}