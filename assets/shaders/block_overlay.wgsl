#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

#import blocks::texture_layer;
#import utils::rgb_to_gray;

@group(2) @binding(0) var overlay_tex: texture_2d<f32>;
@group(2) @binding(1) var overlay_smp: sampler;
@group(2) @binding(2) var blocks_tex: texture_2d_array<f32>;
@group(2) @binding(3) var blocks_smp: sampler;
@group(2) @binding(4) var<uniform> block_id: u32;

struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3f,
    @location(1) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) layer: u32,
    @location(2) color: vec4f
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let direction = vertex.vertex_index / 4;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    out.clip_position.z += 0.0001;
    out.uv = vertex.uv;
    out.layer = texture_layer(block_id, direction);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    var overlay_color = textureSample(overlay_tex, overlay_smp, in.uv);
    var blocks_color = textureSample(blocks_tex, blocks_smp, in.uv, in.layer);

    let gray = rgb_to_gray(blocks_color.xyz);
    return mix(overlay_color, vec4f(vec3f(1.0 - gray), 1.0), overlay_color.a);
}
