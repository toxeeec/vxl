use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};

// xxxxxxxxxxxx | xx       | xxx       | xxxxxxx | xxxx | xxxx
//              | block id | direction | y       | z    | x
pub(super) const ATTRIBUTE_BLOCK_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("Data", 1000000, VertexFormat::Sint32);

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(super) struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    offset: IVec2,
}

impl ChunkMaterial {
    pub(super) fn new(offset: IVec2, texture: Handle<Image>) -> Self {
        Self { offset, texture }
    }
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
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ATTRIBUTE_BLOCK_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
