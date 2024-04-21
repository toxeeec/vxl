#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

const chunk_width = 16u;
const chunk_height = 128u;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
	@builtin(vertex_index) vertex_index: u32,
	@location(0) data: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var block_vertices = array<array<vec3u, 4>, 6>(
        array<vec3u, 4>(vec3u(1, 1, 0), vec3u(0, 1, 0), vec3u(0, 0, 0), vec3u(1, 0, 0)), // north (-z)
        array<vec3u, 4>(vec3u(1, 1, 1), vec3u(1, 1, 0), vec3u(1, 0, 0), vec3u(1, 0, 1)), // east  (+x)
        array<vec3u, 4>(vec3u(0, 1, 1), vec3u(1, 1, 1), vec3u(1, 0, 1), vec3u(0, 0, 1)), // south (+z)
        array<vec3u, 4>(vec3u(0, 1, 0), vec3u(0, 1, 1), vec3u(0, 0, 1), vec3u(0, 0, 0)), // west  (-x)
        array<vec3u, 4>(vec3u(0, 1, 0), vec3u(1, 1, 0), vec3u(1, 1, 1), vec3u(0, 1, 1)), // up    (-y)
        array<vec3u, 4>(vec3u(0, 0, 1), vec3u(1, 0, 1), vec3u(1, 0, 0), vec3u(0, 0, 0)), // down  (+y)
    );

    var out: VertexOutput;

    let x = vertex.data & (chunk_width - 1);
    let z = (vertex.data >> u32(log2(f32(chunk_width)))) & (chunk_width - 1);
    let y = (vertex.data >> u32(log2(f32(chunk_width)) * 2)) & (chunk_height - 1);
    let direction = (vertex.data >> u32(log2(f32(chunk_width)) * 2 + log2(f32(chunk_height)))) & 7;

    let vertex_pos = block_vertices[direction][vertex.vertex_index & 3];

    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4f(
            f32(x + vertex_pos.x),
            f32(y + vertex_pos.y),
            f32(z + vertex_pos.z),
            1.0
        ),
    );

    return out;
}

@fragment
fn fragment() -> @location(0) vec4f {
    return vec4(1.0, 0.0, 0.0, 1.0);
}
