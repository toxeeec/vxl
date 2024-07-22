use bevy::{
    asset::LoadState,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef},
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
    },
};

use crate::{sets::LoadingSet, state::AppState};

// xxxxxxxxxxxx | xx       | xxx       | xxxxxxx | xxxx | xxxx
//              | block id | direction | y       | z    | x
pub(super) const ATTRIBUTE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("Data", 1000000, VertexFormat::Sint32);

#[derive(Resource, Debug)]
struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(super) struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    offset: IVec2,
}

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkTexture(pub(super) Handle<Image>);

#[derive(Debug)]
pub(super) struct ChunkMaterialPlugin;

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
            .get_layout(&[ATTRIBUTE_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

impl ChunkTexture {
    pub(super) fn new(handle: Handle<Image>) -> Self {
        Self(handle)
    }
}

impl Plugin for ChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_systems(OnEnter(AppState::Loading), Self::setup_loading_texture)
            .add_systems(Update, (Self::create_texture).in_set(LoadingSet));
    }
}

impl ChunkMaterialPlugin {
    const TEXTURE_SIZE: usize = 8;

    pub(super) fn is_loaded(texture: Option<Res<ChunkTexture>>) -> bool {
        texture.is_some()
    }

    fn setup_loading_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.insert_resource(LoadingTexture {
            is_loaded: false,
            handle: asset_server.load("textures/blocks.png"),
        });
    }

    fn create_texture(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut loading_texture: ResMut<LoadingTexture>,
        mut images: ResMut<Assets<Image>>,
    ) {
        if loading_texture.is_loaded
            || asset_server.load_state(loading_texture.handle.id()) != LoadState::Loaded
        {
            return;
        }
        loading_texture.is_loaded = true;

        let image = images.get_mut(&loading_texture.handle).unwrap();

        let width = image.width() as usize;
        let height = image.height() as usize;
        let columns = width / Self::TEXTURE_SIZE;
        let rows = height / Self::TEXTURE_SIZE;
        let layers = columns * rows;
        let components = image.texture_descriptor.format.components() as usize;

        let mut data = Vec::with_capacity(image.data.len());
        for i in 0..(rows * columns * Self::TEXTURE_SIZE) {
            let scanline = i % Self::TEXTURE_SIZE;
            let x = (i / Self::TEXTURE_SIZE) % columns;
            let y = (i / Self::TEXTURE_SIZE) / columns;

            let offset =
                (width * (y * Self::TEXTURE_SIZE + scanline) + x * Self::TEXTURE_SIZE) * components;
            data.extend_from_slice(&image.data[offset..(offset + Self::TEXTURE_SIZE * components)]);
        }

        image.data = data;
        image.reinterpret_size(Extent3d {
            width: Self::TEXTURE_SIZE as u32,
            height: Self::TEXTURE_SIZE as u32,
            depth_or_array_layers: layers as u32,
        });

        commands.insert_resource(ChunkTexture::new(loading_texture.handle.clone()));
    }
}
