#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var background_tex: texture_2d<f32>;
@group(1) @binding(1) var background_smp: sampler;
@group(1) @binding(2) var crosshair_tex: texture_2d<f32>;
@group(1) @binding(3) var crosshair_smp: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
};

fn rgb_to_gray(rgb: vec3f) -> f32 {
    return rgb.r * 0.299 + rgb.g * 0.587 + rgb.b * 0.114;
}

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) vertex_position: vec3f
) -> VertexOutput {
    var uvs = array<vec2f, 4>(vec2f(0, 0), vec2f(1, 0), vec2f(1, 1), vec2f(0, 1));
    let center = view.viewport.zw / 4;

    var out: VertexOutput;

    out.clip_position = view.clip_from_world * vec4(vertex_position.xy + center, vertex_position.z, 1.0);
    out.uv = uvs[vertex_index];

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    var background_color = textureSample(background_tex, background_smp, in.uv);
    var crosshair_color = textureSample(crosshair_tex, crosshair_smp, in.uv);

    let gray = rgb_to_gray(background_color.xyz);
    return mix(background_color, vec4f(vec3f(1.0 - gray), 1.0), crosshair_color.a);
}
