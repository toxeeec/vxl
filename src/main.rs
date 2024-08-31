#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use block_overlay::BlockOverlayPlugin;
use camera::CameraPlugin;
use crosshair::CrosshairPlugin;
use diagnostics::DiagnosticsPlugin;
use materials::{BlockOverlayMaterial, ChunkMaterial};
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use sets::{GameplaySet, LoadingSet};
use state::AppState;
use textures::TexturesPlugin;
use toml_asset::{TomlAsset, TomlLoader};
use world::WorldPlugin;

mod block;
mod block_overlay;
mod camera;
mod crosshair;
mod diagnostics;
mod direction;
mod materials;
mod physics;
mod player;
mod sets;
mod settings;
mod state;
mod textures;
mod toml_asset;
mod world;

pub const UTILS_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(131465340603768031182789503363035378691);
pub const DIRECTION_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(40560788717271163742317989904110872029);
pub const BLOCKS_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(5216533342730733270932431900685248162);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_asset::<TomlAsset>()
        .init_asset_loader::<TomlLoader>()
        .add_plugins((
            BlockOverlayPlugin,
            CameraPlugin,
            CrosshairPlugin,
            DiagnosticsPlugin,
            MaterialPlugin::<BlockOverlayMaterial>::default(),
            MaterialPlugin::<ChunkMaterial>::default(),
            PhysicsPlugin,
            PlayerPlugin,
            TexturesPlugin,
            WorldPlugin,
        ))
        .init_state::<AppState>()
        .configure_sets(FixedUpdate, GameplaySet.run_if(in_state(AppState::InGame)))
        .configure_sets(
            Update,
            (
                GameplaySet.run_if(in_state(AppState::InGame)),
                LoadingSet.run_if(in_state(AppState::Loading)),
            ),
        )
        .insert_resource(Msaa::Off)
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(
            Update,
            start_generating
                .run_if(in_state(AppState::Loading))
                .run_if(TexturesPlugin::is_loaded)
                .run_if(WorldPlugin::is_loaded),
        )
        .add_systems(
            Update,
            start_game
                .run_if(in_state(AppState::Generating))
                .run_if(WorldPlugin::is_generated),
        );

    load_internal_asset!(app, UTILS_HANDLE, "utils.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, DIRECTION_HANDLE, "direction.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, BLOCKS_HANDLE, "blocks.wgsl", Shader::from_wgsl);

    app.run();
}

fn setup(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = query.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn start_generating(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Generating);
}

fn start_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}
