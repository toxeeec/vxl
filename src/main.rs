#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod block;
mod camera;
mod chunk;
mod debug;
mod direction;
mod physics;
mod player;
mod position;
mod settings;
mod texture;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::CursorGrabMode,
};
use camera::{CameraMovement, CameraPlugin};
use chunk::ChunkPlugin;
use debug::DebugPlugin;
use leafwing_input_manager::{axislike::DualAxis, input_map::InputMap, InputManagerBundle};
use physics::PhysicsPlugin;
use player::{Player, PlayerBundle, PlayerPlugin};
use texture::TexturePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            PhysicsPlugin,
            TexturePlugin,
            CameraPlugin,
            PlayerPlugin,
            ChunkPlugin,
            DebugPlugin,
        ))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, (setup, spawn_debug_text))
        .add_systems(Update, display_debug_text)
        .run();
}

#[derive(Component, Debug)]
struct DebugText;

fn setup(mut commands: Commands, mut query: Query<&mut Window>) {
    let mut window = query.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    commands
        .spawn(PlayerBundle::new(Transform::from_xyz(0.0, 100.0, 0.0)))
        .with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..Default::default()
                    }
                    .into(),
                    tonemapping: Tonemapping::None,
                    transform: Transform::from_xyz(0.0, 1.6, 0.0),
                    ..Default::default()
                },
                InputManagerBundle::<CameraMovement> {
                    input_map: InputMap::default()
                        .insert(DualAxis::mouse_motion(), CameraMovement::Rotation)
                        .build(),
                    ..Default::default()
                },
            ));
        });
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
    mut q_text: Query<&mut Text, With<DebugText>>,
    q_pos: Query<&Transform, With<Player>>,
    diagnostics: Res<DiagnosticsStore>,
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
