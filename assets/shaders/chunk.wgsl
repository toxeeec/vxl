#import bevy_pbr::forward_io::VertexOutput

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, smp, mesh.uv);
}
