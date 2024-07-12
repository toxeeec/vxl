use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    physics::{Acceleration, MovementBundle, PhysicalPosition, PhysicsSet, RigidBody},
    settings,
    world::CHUNK_WIDTH,
};

#[derive(Component, Default, Debug)]
pub(super) struct Player;

#[derive(Event, Debug)]
pub(super) struct PlayerSpawnEvent {
    pub(super) offset: IVec2,
}

#[derive(Event, Debug)]
pub(super) struct PlayerChunkMoveEvent {
    pub(super) new_offset: IVec2,
}

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
    rigid_body: RigidBody,
}

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl PlayerSpawnEvent {
    pub(super) fn new(offset: IVec2) -> Self {
        Self { offset }
    }
}
impl PlayerChunkMoveEvent {
    pub(super) fn new(new_offset: IVec2) -> Self {
        Self { new_offset }
    }
}

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
            rigid_body: RigidBody::new(0.6, 1.8),
            ..Default::default()
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerChunkMoveEvent>()
            .add_plugins((
                InputManagerPlugin::<CameraAction>::default(),
                InputManagerPlugin::<MovementAction>::default(),
            ))
            .add_systems(Startup, Self::setup)
            .add_systems(FixedUpdate, Self::player_chunk_move.after(PhysicsSet))
            .add_systems(
                Update,
                (Self::turn_player, Self::handle_player_movement).chain(),
            );
    }
}

impl PlayerPlugin {
    const ACCELERATION: f32 = 0.35;

    fn setup(mut commands: Commands, mut events: EventWriter<PlayerSpawnEvent>) {
        let pos = Vec3::new(0.0, 60.0, 0.0);
        commands.spawn(PlayerBundle::new(Transform::from_translation(pos)));
        events.send(PlayerSpawnEvent::new(
            pos.xz()
                .as_ivec2()
                .div_euclid(IVec2::splat(CHUNK_WIDTH as i32)),
        ));
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
        mut query: Query<
            (&Transform, &ActionState<MovementAction>, &mut Acceleration),
            With<Player>,
        >,
    ) {
        let (transform, action_state, mut acceleration) = query.single_mut();

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

        *acceleration = Acceleration::new(direction.normalize_or_zero() * Self::ACCELERATION);
    }

    fn player_chunk_move(
        query: Query<&PhysicalPosition, With<Player>>,
        mut events: EventWriter<PlayerChunkMoveEvent>,
    ) {
        let player = query.single();

        let curr_offset = player
            .current()
            .xz()
            .as_ivec2()
            .div_euclid(IVec2::splat(CHUNK_WIDTH as i32));

        let Some(prev) = player.previous() else {
            events.send(PlayerChunkMoveEvent::new(curr_offset));
            return;
        };

        let prev_offset = prev
            .xz()
            .as_ivec2()
            .div_euclid(IVec2::splat(CHUNK_WIDTH as i32));
        if curr_offset != prev_offset {
            events.send(PlayerChunkMoveEvent::new(curr_offset));
        }
    }
}
