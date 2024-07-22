#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{prelude::*, window::CursorGrabMode};
use camera::CameraPlugin;
use diagnostics::DiagnosticsPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use sets::{GameplaySet, LoadingSet};
use state::AppState;
use texture::ChunkMaterialPlugin;
use toml_asset::{TomlAsset, TomlLoader};
use world::WorldPlugin;

mod block;
mod camera;
mod diagnostics;
mod direction;
mod physics;
mod player;
mod sets;
mod settings;
mod state;
mod texture;
mod toml_asset;
mod world;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            CameraPlugin,
            ChunkMaterialPlugin,
            DiagnosticsPlugin,
            PhysicsPlugin,
            PlayerPlugin,
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
        .init_asset::<TomlAsset>()
        .init_asset_loader::<TomlLoader>()
        .insert_resource(Msaa::Off)
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(
            Update,
            start_generating
                .run_if(in_state(AppState::Loading))
                .run_if(ChunkMaterialPlugin::is_loaded)
                .run_if(WorldPlugin::is_loaded),
        )
        .add_systems(
            Update,
            start_game
                .run_if(in_state(AppState::Generating))
                .run_if(WorldPlugin::is_generated),
        )
        .run();
}

fn setup(mut query: Query<&mut Window>) {
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
