mod systems;

pub(crate) use systems::set_player_velocity;

use crate::{camera::rotate_camera, chunk::CenterOffset, physics::LinearVelocity};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use systems::{rotate_player, update_center_offset};

#[derive(Component, Default, Debug)]
pub(super) struct Player {
    pub(super) rotation: f32,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Reflect, Debug)]
pub(super) enum PlayerAction {
    Forward,
    Right,
    Backward,
    Left,
    Up,
    Down,
}

#[derive(Bundle)]
pub(super) struct PlayerBundle {
    player: Player,
    transform: TransformBundle,
    velocity: LinearVelocity,
    player_input_manager: InputManagerBundle<PlayerAction>,
}

impl PlayerBundle {
    pub(super) fn new(transform: Transform) -> Self {
        Self {
            transform: transform.into(),
            player_input_manager: InputManagerBundle::<PlayerAction> {
                input_map: InputMap::new([
                    (QwertyScanCode::W, PlayerAction::Forward),
                    (QwertyScanCode::A, PlayerAction::Left),
                    (QwertyScanCode::S, PlayerAction::Backward),
                    (QwertyScanCode::D, PlayerAction::Right),
                    (QwertyScanCode::Space, PlayerAction::Up),
                    (QwertyScanCode::ShiftLeft, PlayerAction::Down),
                ]),
                ..Default::default()
            },
            player: Player::default(),
            velocity: LinearVelocity::ZERO,
        }
    }
}

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<CenterOffset>()
            .add_systems(
                Update,
                (
                    (rotate_camera, rotate_player, set_player_velocity).chain(),
                    update_center_offset,
                ),
            );
    }
}
