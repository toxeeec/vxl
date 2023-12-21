#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>, 
	@location(1) uv: vec2<f32>,
	@location(2) direction: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
	@location(1) light: f32,
};

fn light_from_direction(dir: u32) -> f32 {
    switch dir {
		case 4u: {
            return 1.0;
        }
		case 0u, 2u: {
            return 0.8;
        }
		case 1u, 3u: {
            return 0.6;
        }
		case 5u: {
            return 0.5;
        }
		default: {
            return 0.0;
        }
	}
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    out.light = light_from_direction(vertex.direction);
    return out;
}

    @fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, smp, in.uv) * in.light;
}
