#![allow(clippy::type_complexity)]

use bevy::{prelude::*, window::CursorGrabMode};
use camera::CameraPlugin;
use diagnostics::DiagnosticsPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

mod camera;
mod diagnostics;
mod physics;
mod player;
mod settings;
mod world;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CameraPlugin,
            DiagnosticsPlugin,
            PhysicsPlugin,
            PlayerPlugin,
            WorldPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut query: Query<&mut Window>) {
    let mut window = query.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}
