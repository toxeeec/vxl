use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::settings;

#[derive(Component, Default, Debug)]
pub(super) struct Player;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Reflect, Debug)]
pub(super) enum PlayerAction {
    Turn,
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    player: Player,
    transform: TransformBundle,
    input_manager: InputManagerBundle<PlayerAction>,
}

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl PlayerBundle {
    fn new(transform: Transform) -> Self {
        Self {
            transform: transform.into(),
            input_manager: InputManagerBundle::with_map(InputMap::new([(
                PlayerAction::Turn,
                DualAxis::mouse_motion(),
            )])),
            ..Default::default()
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(Startup, Self::setup)
            .add_systems(Update, Self::turn_player);
    }
}

impl PlayerPlugin {
    fn setup(mut commands: Commands) {
        commands.spawn(PlayerBundle::new(Transform::from_xyz(0.5, 0.0, 5.0)));
    }

    fn turn_player(mut query: Query<(&mut Transform, &ActionState<PlayerAction>), With<Player>>) {
        let (mut player, action_state) = query.single_mut();
        let delta = action_state.axis_pair(&PlayerAction::Turn).unwrap().x();
        if delta == 0.0 {
            return;
        }
        let (mut yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
        yaw -= delta.to_radians() * settings::SENSITIVITY;

        player.rotation = Quat::from_rotation_y(yaw);
    }
}
