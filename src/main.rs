#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod block;
mod chunk;
mod direction;
mod player;
mod settings;
mod texture;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::CursorGrabMode,
};
use chunk::ChunkPlugin;
use player::PlayerPlugin;
use texture::TexturePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            TexturePlugin,
            ChunkPlugin,
            PlayerPlugin,
        ))
        .add_systems(Startup, (setup, spawn_debug_text))
        .add_systems(Update, display_debug_text)
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Component, Debug)]
struct DebugText;

fn setup(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn spawn_debug_text(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 24.0,
        ..Default::default()
    };

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("FPS: ", text_style.clone()),
            TextSection::from_style(text_style.clone()),
            TextSection::new("X/Y/Z: ", text_style.clone()),
            TextSection::from_style(text_style.clone()),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..Default::default()
        }),
        DebugText,
    ));
}

fn display_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    q_pos: Query<&Transform, With<Camera>>,
    mut q_text: Query<&mut Text, With<DebugText>>,
) {
    let mut text = q_text.single_mut();
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps.smoothed() {
            text.sections[1].value = format!("{fps:.0}\n");
        }
    }
    let Vec3 { x, y, z } = q_pos.single().translation;
    text.sections[3].value = format!("{x:.3}/{y:.3}/{z:.3}\n");
}
