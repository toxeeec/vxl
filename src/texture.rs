use crate::block::BlockId;
use crate::direction::Direction;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

#[derive(Resource, Debug)]
pub(crate) struct ChunkTexture {
    pub(crate) material: Handle<ChunkMaterial>,
    pub(crate) atlas: Handle<TextureAtlas>,
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
#[bind_group_data(ChunkMaterialKey)]
pub(crate) struct ChunkMaterial {
    #[texture(0, dimension = "2d")]
    #[sampler(1)]
    pub(crate) texture: Handle<Image>,
    pub(crate) wireframe_mode: bool,
}

impl FromWorld for ChunkTexture {
    fn from_world(world: &mut World) -> Self {
        let image = world.resource::<AssetServer>().load("blocks.png");
        let texture_atlas =
            TextureAtlas::from_grid(image, Vec2 { x: 8.0, y: 8.0 }, 8, 8, None, None);
        let material = world
            .resource_mut::<Assets<ChunkMaterial>>()
            .add(ChunkMaterial {
                texture: texture_atlas.texture.clone(),
                wireframe_mode: false,
            });
        let atlas = world
            .resource_mut::<Assets<TextureAtlas>>()
            .add(texture_atlas);

        Self { material, atlas }
    }
}

impl Material for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/chunk.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = if key.bind_group_data.wireframe_mode {
            PolygonMode::Line
        } else {
            PolygonMode::Fill
        };
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ChunkMaterialKey {
    wireframe_mode: bool,
}

impl From<&ChunkMaterial> for ChunkMaterialKey {
    fn from(material: &ChunkMaterial) -> Self {
        Self {
            wireframe_mode: material.wireframe_mode,
        }
    }
}

pub(crate) fn atlas_uvs(atlas: &TextureAtlas, block_id: BlockId, dir: Direction) -> [[f32; 2]; 4] {
    let area = atlas.textures[block_id.texture_id(dir)];
    let Vec2 { x: min_u, y: min_v } = (area.min + area.size() / 2.0 / atlas.size) / atlas.size;
    let Vec2 { x: max_u, y: max_v } = (area.max - area.size() / 2.0 / atlas.size) / atlas.size;

    [
        [max_u, min_v], // top-right
        [min_u, min_v], // top-left
        [min_u, max_v], // bottom-left
        [max_u, max_v], // bottom-right
    ]
}

#[derive(Debug)]
pub(super) struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .init_resource::<ChunkTexture>();
    }
}
