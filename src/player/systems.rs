use super::bundles::PlayerBundle;
use super::Player;
use crate::settings::{PLAYER_SPEED, SENSITIVITY};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub(super) fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::new(Vec3::new(0.0, 8.0, 0.0)));
}

pub(super) fn player_movement(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::W) {
        direction += transform.forward();
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += transform.back();
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction += transform.left();
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += transform.right();
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction += Vec3::NEG_Y;
    }

    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    transform.translation += movement;
}

pub(super) fn player_rotation(
    mut query: Query<&mut Transform, With<Player>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let mut transform = query.single_mut();
    let rotation = transform.rotation;
    let (mut yaw, mut pitch, _) = rotation.to_euler(EulerRot::YXZ);

    let delta = motion_evr.read().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
    pitch -= delta.y.to_radians() * SENSITIVITY;
    yaw -= delta.x.to_radians() * SENSITIVITY;
    pitch = pitch.clamp(-89.9f32.to_radians(), 89.9f32.to_radians());

    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}
