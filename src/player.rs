use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    physics::{MovementBundle, PhysicalPosition, SetAccelerationEvent},
    settings,
    world::CHUNK_WIDTH,
};

#[derive(Component, Default, Debug)]
pub(super) struct Player;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Reflect, Debug)]
pub(super) enum CameraAction {
    Turn,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Reflect, Debug)]
enum MovementAction {
    Forward,
    Right,
    Backward,
    Left,
    Up,
    Down,
    Sprint,
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    player: Player,
    transform: TransformBundle,
    camera_action_manager: InputManagerBundle<CameraAction>,
    movement_action_manager: InputManagerBundle<MovementAction>,
    physical_position: PhysicalPosition,
    movement_bundle: MovementBundle,
}

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl PlayerBundle {
    fn new(transform: Transform) -> Self {
        Self {
            transform: transform.into(),
            camera_action_manager: InputManagerBundle::with_map(InputMap::new([(
                CameraAction::Turn,
                DualAxis::mouse_motion(),
            )])),
            movement_action_manager: InputManagerBundle::with_map(InputMap::new([
                (MovementAction::Forward, KeyCode::KeyW),
                (MovementAction::Left, KeyCode::KeyA),
                (MovementAction::Backward, KeyCode::KeyS),
                (MovementAction::Right, KeyCode::KeyD),
                (MovementAction::Up, KeyCode::Space),
                (MovementAction::Down, KeyCode::ShiftLeft),
            ])),
            physical_position: transform.into(),
            ..Default::default()
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<CameraAction>::default(),
            InputManagerPlugin::<MovementAction>::default(),
        ))
        .add_systems(Startup, Self::setup)
        .add_systems(
            Update,
            (Self::turn_player, Self::handle_player_movement).chain(),
        );
    }
}

impl PlayerPlugin {
    const ACCELERATION: f32 = 0.35;

    fn setup(mut commands: Commands) {
        let half_chunk = CHUNK_WIDTH as f32 / 2.0;
        commands.spawn(PlayerBundle::new(Transform::from_xyz(
            half_chunk,
            CHUNK_WIDTH as f32,
            half_chunk,
        )));
    }

    fn turn_player(mut query: Query<(&mut Transform, &ActionState<CameraAction>), With<Player>>) {
        let (mut player, action_state) = query.single_mut();
        let delta = action_state.axis_pair(&CameraAction::Turn).unwrap().x();
        if delta == 0.0 {
            return;
        }

        let (mut yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
        yaw -= delta.to_radians() * settings::SENSITIVITY;

        player.rotation = Quat::from_rotation_y(yaw);
    }

    fn handle_player_movement(
        mut query: Query<(Entity, &Transform, &ActionState<MovementAction>), With<Player>>,
        mut events: EventWriter<SetAccelerationEvent>,
    ) {
        let (entity, transform, action_state) = query.single_mut();

        let mut direction = Vec3::ZERO;

        if action_state.pressed(&MovementAction::Forward) {
            direction += *transform.forward();
        }
        if action_state.pressed(&MovementAction::Backward) {
            direction += *transform.back();
        }
        if action_state.pressed(&MovementAction::Left) {
            direction += *transform.left();
        }
        if action_state.pressed(&MovementAction::Right) {
            direction += *transform.right();
        }
        if action_state.pressed(&MovementAction::Up) {
            direction += *transform.up();
        }
        if action_state.pressed(&MovementAction::Down) {
            direction += *transform.down();
        }

        events.send(SetAccelerationEvent::new(
            entity,
            direction.normalize_or_zero() * Self::ACCELERATION,
        ));
    }
}
