use bevy::{
    asset::LoadState,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::Extent3d,
    },
};

use crate::sets::LoadingSet;

#[derive(Resource, Debug)]
pub(super) struct LoadingTextures {
    blocks: Handle<Image>,
    blocks_loaded: bool,
    crosshair: Handle<Image>,
    crosshair_loaded: bool,
}

#[derive(Resource, Debug)]
pub(super) struct BlocksTexture(pub(super) Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Debug)]
pub(super) struct CrosshairTexture(pub(super) Handle<Image>);

#[derive(Debug)]
pub(super) struct TexturesPlugin;

impl FromWorld for LoadingTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        LoadingTextures {
            blocks: asset_server.load("textures/blocks.png"),
            blocks_loaded: false,
            crosshair: asset_server.load("textures/crosshair.png"),
            crosshair_loaded: false,
        }
    }
}

impl Plugin for TexturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingTextures>()
            .add_plugins(ExtractResourcePlugin::<CrosshairTexture>::default())
            .add_systems(
                Update,
                (
                    (
                        Self::create_blocks_array_texture,
                        Self::create_crosshair_texture,
                    ),
                    Self::update_loaded_state,
                )
                    .chain()
                    .in_set(LoadingSet),
            );
    }
}

impl TexturesPlugin {
    const BLOCKS_TEXTURE_TILE_SIZE: usize = 8;

    pub(super) fn is_loaded(loading_textures: Res<LoadingTextures>) -> bool {
        loading_textures.blocks_loaded && loading_textures.crosshair_loaded
    }

    fn update_loaded_state(
        asset_server: Res<AssetServer>,
        mut loading_textures: ResMut<LoadingTextures>,
    ) {
        if asset_server.load_state(loading_textures.blocks.id()) == LoadState::Loaded {
            loading_textures.blocks_loaded = true;
        }

        if asset_server.load_state(loading_textures.crosshair.id()) == LoadState::Loaded {
            loading_textures.crosshair_loaded = true;
        }
    }

    fn create_blocks_array_texture(
        mut commands: Commands,
        loading_textures: Res<LoadingTextures>,
        blocks: Option<Res<BlocksTexture>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        if !loading_textures.blocks_loaded || blocks.is_some() {
            return;
        }

        let image = images.get_mut(&loading_textures.blocks).unwrap();

        let width = image.width() as usize;
        let height = image.height() as usize;
        let columns = width / Self::BLOCKS_TEXTURE_TILE_SIZE;
        let rows = height / Self::BLOCKS_TEXTURE_TILE_SIZE;
        let layers = columns * rows;
        let components = image.texture_descriptor.format.components() as usize;

        let mut data = Vec::with_capacity(image.data.len());
        for i in 0..(rows * columns * Self::BLOCKS_TEXTURE_TILE_SIZE) {
            let scanline = i % Self::BLOCKS_TEXTURE_TILE_SIZE;
            let x = (i / Self::BLOCKS_TEXTURE_TILE_SIZE) % columns;
            let y = (i / Self::BLOCKS_TEXTURE_TILE_SIZE) / columns;

            let offset = (width * (y * Self::BLOCKS_TEXTURE_TILE_SIZE + scanline)
                + x * Self::BLOCKS_TEXTURE_TILE_SIZE)
                * components;
            data.extend_from_slice(
                &image.data[offset..(offset + Self::BLOCKS_TEXTURE_TILE_SIZE * components)],
            );
        }

        image.data = data;
        image.reinterpret_size(Extent3d {
            width: Self::BLOCKS_TEXTURE_TILE_SIZE as u32,
            height: Self::BLOCKS_TEXTURE_TILE_SIZE as u32,
            depth_or_array_layers: layers as u32,
        });

        commands.insert_resource(BlocksTexture(loading_textures.blocks.clone()));
    }

    fn create_crosshair_texture(
        mut commands: Commands,
        loading_textures: Res<LoadingTextures>,
        crosshair: Option<Res<CrosshairTexture>>,
    ) {
        if !loading_textures.crosshair_loaded || crosshair.is_some() {
            return;
        }

        commands.insert_resource(CrosshairTexture(loading_textures.crosshair.clone()));
    }
}
