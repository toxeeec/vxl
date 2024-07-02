#![allow(clippy::type_complexity)]

use bevy::{prelude::*, window::CursorGrabMode};
use camera::CameraPlugin;
use diagnostics::DiagnosticsPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use texture::ChunkMaterialPlugin;
use toml_asset::{TomlAsset, TomlLoader};
use world::WorldPlugin;

mod block;
mod camera;
mod diagnostics;
mod direction;
mod physics;
mod player;
mod settings;
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
        .init_asset::<TomlAsset>()
        .init_asset_loader::<TomlLoader>()
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut query: Query<&mut Window>) {
    let mut window = query.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}
