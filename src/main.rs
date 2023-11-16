#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod block;
mod chunk;
mod direction;
mod settings;
mod texture;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::*,
    window::CursorGrabMode,
};
use chunk::ChunkPlugin;
use settings::{PLAYER_SPEED, SENSITIVITY};
use texture::TexturePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            TexturePlugin,
            ChunkPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (camera_movement, camera_rotation, display_debug_text),
        )
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Component, Debug)]
struct DebugText;

fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.spawn(Camera3dBundle {
        tonemapping: Tonemapping::None,
        transform: Transform::from_xyz(0.0, 8.0, 0.0).looking_at(Vec3::splat(6.0), Vec3::Y),
        ..Default::default()
    });

    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

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

fn camera_movement(
    mut query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut camera_transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::W) {
        direction += camera_transform.forward();
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += camera_transform.back();
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction += camera_transform.left();
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += camera_transform.right();
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction += Vec3::NEG_Y;
    }

    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    camera_transform.translation += movement;
}

fn camera_rotation(
    mut query: Query<&mut Transform, With<Camera>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let mut camera_transform = query.single_mut();
    let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    let delta = motion_evr.iter().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
    pitch -= delta.y.to_radians() * SENSITIVITY;
    yaw -= delta.x.to_radians() * SENSITIVITY;
    pitch = pitch.clamp(-89.9f32.to_radians(), 89.9f32.to_radians());

    camera_transform.rotation =
        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
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
