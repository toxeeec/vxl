mod systems;

use crate::camera::CameraMovement;
use crate::chunk::CenterOffset;
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use leafwing_input_manager::prelude::*;
pub(crate) use systems::move_player;
use systems::spawn_player;

#[derive(Component, Debug)]
pub(super) struct Player;

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
struct PlayerBundle {
    camera: Camera3dBundle,
    camera_input_manager: InputManagerBundle<CameraMovement>,
    player_input_manager: InputManagerBundle<PlayerAction>,
    player: Player,
}

impl PlayerBundle {
    fn new(transform: Transform) -> Self {
        Self {
            player: Player,
            camera: Camera3dBundle {
                tonemapping: Tonemapping::None,
                transform,
                ..Default::default()
            },
            camera_input_manager: InputManagerBundle::<CameraMovement> {
                input_map: InputMap::default()
                    .insert(DualAxis::mouse_motion(), CameraMovement::Rotation)
                    .build(),
                ..Default::default()
            },
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
        }
    }
}

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<CenterOffset>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}
