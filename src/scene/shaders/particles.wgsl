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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    particle: ParticleInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;
    let world_position = vec4<f32>(vertex.position*particle.size + particle.position, 0.0, 1.0);
    out.clip_position = uniforms.projection * world_position;
    return out;
}

// Fragment shader

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}