#import bevy_pbr::mesh_vertex_output MeshVertexOutput

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, smp, mesh.uv);
}
