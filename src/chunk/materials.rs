use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, TypePath, Clone, Debug)]
#[uuid = "0993d972-901b-485c-b9f1-00e8d1549724"]
pub(crate) struct ChunkMaterial {
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    pub(super) texture: Handle<Image>,
}

impl Material for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }
}
