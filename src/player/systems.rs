use super::{Player, PlayerAction};
use crate::chunk::CenterOffset;
use crate::physics::LinearVelocity;
use crate::position::Offset;
use crate::settings::PLAYER_SPEED;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(crate) fn rotate_player(
    mut q_player: Query<&mut Player>,
    q_camera: Query<&Transform, With<Camera>>,
) {
    let mut player = q_player.single_mut();
    let camera = q_camera.single();

    player.rotation = camera.rotation.to_euler(EulerRot::YXZ).0;
}

pub(crate) fn set_player_velocity(
    mut q_player: Query<(&Player, &mut LinearVelocity)>,
    q_action: Query<&ActionState<PlayerAction>>,
) {
    let (&Player { rotation }, mut velocity) = q_player.single_mut();
    let action_state = q_action.single();
    let mut new_velocity = LinearVelocity::ZERO;

    let direction = Vec3::new(-rotation.sin(), 0.0, -rotation.cos());

    if action_state.pressed(PlayerAction::Forward) {
        new_velocity += direction;
    }
    if action_state.pressed(PlayerAction::Backward) {
        new_velocity -= direction;
    }
    if action_state.pressed(PlayerAction::Left) {
        new_velocity -= direction.cross(Vec3::Y);
    }
    if action_state.pressed(PlayerAction::Right) {
        new_velocity += direction.cross(Vec3::Y);
    }
    if action_state.pressed(PlayerAction::Up) {
        new_velocity += Vec3::Y;
    }
    if action_state.pressed(PlayerAction::Down) {
        new_velocity += Vec3::NEG_Y;
    }

    new_velocity = new_velocity.normalize_or_zero() * PLAYER_SPEED;
    if new_velocity != *velocity {
        *velocity = new_velocity;
    }
}

pub(crate) fn update_center_offset(
    query: Query<&Transform, (Changed<Transform>, With<Player>)>,
    mut center_offset: ResMut<CenterOffset>,
) {
    match query.get_single() {
        Ok(transform) => {
            let player_offset = Offset::from(transform);
            if player_offset != center_offset.0 {
                *center_offset = CenterOffset(player_offset);
            }
        }
        Err(_) => debug_assert!(query.is_empty()),
    }
}
