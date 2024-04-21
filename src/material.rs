use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct ChunkMaterial {}

impl ChunkMaterial {
    // xxxxxxxxxxxxxx | xxx       | xxxxx | xxxxx | xxxxx
    //                | direction | y     | z     | x
    pub(super) const ATTRIBUTE_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("Data", 1000000, VertexFormat::Uint32);
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[Self::ATTRIBUTE_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
