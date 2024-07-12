#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

const block_grass = 1u;
const block_dirt = 2u;
const block_stone = 3u;

const direction_north = 0u;
const direction_east = 1u;
const direction_south = 2u;
const direction_west = 3u;
const direction_up = 4u;
const direction_down = 5u;

const chunk_width = 16i;
const chunk_height = 256i;

@group(2) @binding(0) var tex: texture_2d_array<f32>;
@group(2) @binding(1) var smp: sampler;
@group(2) @binding(2) var<uniform> offset: vec2i;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
    @location(0) data: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) layer: u32,
    @location(2) brightness: f32,
};

fn texture_layer(block_id: u32, direction: u32) -> u32 {
    switch block_id {
        case block_grass: {
            switch direction {
                case direction_up: { return 0u; }
                case direction_down: { return 2u; }
                default: { return 1u; }
            }
        }
        case block_dirt: { return 2u; }
        case block_stone: { return 3u; }
        default: { return u32(-1i); }
    }
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var block_vertices = array<array<vec3i, 4>, 6>(
        array<vec3i, 4>(vec3i(1, 1, 0), vec3i(0, 1, 0), vec3i(0, 0, 0), vec3i(1, 0, 0)), // north (-z)
        array<vec3i, 4>(vec3i(1, 1, 1), vec3i(1, 1, 0), vec3i(1, 0, 0), vec3i(1, 0, 1)), // east  (+x)
        array<vec3i, 4>(vec3i(0, 1, 1), vec3i(1, 1, 1), vec3i(1, 0, 1), vec3i(0, 0, 1)), // south (+z)
        array<vec3i, 4>(vec3i(0, 1, 0), vec3i(0, 1, 1), vec3i(0, 0, 1), vec3i(0, 0, 0)), // west  (-x)
        array<vec3i, 4>(vec3i(0, 1, 0), vec3i(1, 1, 0), vec3i(1, 1, 1), vec3i(0, 1, 1)), // up    (-y)
        array<vec3i, 4>(vec3i(0, 0, 1), vec3i(1, 0, 1), vec3i(1, 0, 0), vec3i(0, 0, 0)), // down  (+y)
    );
    var uvs = array<vec2f, 4>(vec2f(1, 0), vec2f(0, 0), vec2f(0, 1), vec2f(1, 1));
    var brightness_levels = array<f32, 6>(0.8, 0.6, 0.8, 0.6, 1.0, 0.5);

    var out: VertexOutput;

    let x = vertex.data & (chunk_width - 1);
    let z = (vertex.data >> u32(log2(f32(chunk_width)))) & (chunk_width - 1);
    let y = (vertex.data >> u32(log2(f32(chunk_width)) * 2)) & (chunk_height - 1);
    let direction = u32((vertex.data >> u32(log2(f32(chunk_width)) * 2 + log2(f32(chunk_height)))) & 7);
    let block_id = u32((vertex.data >> u32(log2(f32(chunk_width)) * 2 + log2(f32(chunk_height))) + 3) & 3);

    let vertex_idx = vertex.vertex_index & 3;
    let vertex_pos = block_vertices[direction][vertex_idx];

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4f(
            f32(x + vertex_pos.x + offset.x * chunk_width),
            f32(y + vertex_pos.y),
            f32(z + vertex_pos.z + offset.y * chunk_width),
            1.0
        ),
    );
    out.uv = uvs[vertex_idx];
    out.layer = texture_layer(block_id, direction);
    out.brightness = brightness_levels[direction];

    return out;
}

struct FragmentInput {
    @location(0) uv: vec2f,
    @location(1) layer: u32,
    @location(2) brightness: f32,
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4f {
    return textureSample(tex, smp, input.uv, input.layer) * input.brightness;
}
