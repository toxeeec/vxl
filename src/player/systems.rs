use super::{Player, PlayerAction, PlayerBundle};
use crate::chunk::CenterOffset;
use crate::position::Offset;
use crate::settings::PLAYER_SPEED;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) fn spawn_player(mut commands: Commands) {
    let transform = Transform::from_xyz(0.0, 8.0, 0.0).looking_at(Vec3::splat(8.0), Vec3::Y);
    commands.spawn(PlayerBundle::new(transform));
}

pub(crate) fn move_player(
    mut q_transform: Query<&mut Transform, With<Player>>,
    q_action: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
    mut center_offset: ResMut<CenterOffset>,
) {
    let mut transform = q_transform.single_mut();
    let action_state = q_action.single();
    let mut direction = Vec3::ZERO;

    if action_state.pressed(PlayerAction::Forward) {
        direction += transform.forward();
    }
    if action_state.pressed(PlayerAction::Backward) {
        direction += transform.back();
    }
    if action_state.pressed(PlayerAction::Left) {
        direction += transform.left();
    }
    if action_state.pressed(PlayerAction::Right) {
        direction += transform.right();
    }
    if action_state.pressed(PlayerAction::Up) {
        direction += Vec3::Y;
    }
    if action_state.pressed(PlayerAction::Down) {
        direction += Vec3::NEG_Y;
    }

    let offset = Offset::from(*transform);

    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    transform.translation += movement;
    let new_offset = Offset::from(*transform);

    if new_offset != offset {
        center_offset.update(new_offset);
    }
}
