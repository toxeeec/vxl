use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use leafwing_input_manager::prelude::*;

use crate::{
    physics::{
        Acceleration, CollisionEvent, Flying, Grounded, MovementBundle, PhysicalPosition,
        PhysicsSet, RigidBody, Velocity,
    },
    sets::GameplaySet,
    settings,
    state::AppState,
    world::CHUNK_WIDTH,
};

#[derive(Component, Default, Debug)]
pub(super) struct Player;

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
        app.add_event::<PlayerChunkMoveEvent>()
            .add_plugins((
                InputManagerPlugin::<CameraAction>::default(),
                InputManagerPlugin::<MovementAction>::default(),
            ))
            .add_systems(OnEnter(AppState::InGame), Self::spawn_player)
            .add_systems(
                FixedUpdate,
                (Self::player_chunk_move.after(PhysicsSet)).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (
                    Self::turn_player,
                    Self::handle_player_horizontal_movement,
                    Self::handle_player_vertical_movement,
                    Self::handle_player_jumping,
                )
                    .chain()
                    .in_set(GameplaySet),
            );
    }
}

impl PlayerPlugin {
    const ACCELERATION: f32 = 64.0;
    const JUMP_VELOCITY: f32 = 10.0;
    const DOUBLE_TAP_DELAY: Duration = Duration::from_millis(500);

    fn spawn_player(mut commands: Commands) {
        let pos = Vec3::new(0.0, 60.0, 0.0);
        commands.spawn(PlayerBundle::new(Transform::from_translation(pos)));
    }

    fn turn_player(mut query: Query<(&mut Transform, &ActionState<CameraAction>), With<Player>>) {
        let (mut player, action_state) = query.single_mut();
        let delta = action_state
            .axis_pair(&CameraAction::Turn)
            .unwrap_or_default()
            .x();
        if delta == 0.0 {
            return;
        }

        let (mut yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
        yaw -= delta.to_radians() * settings::SENSITIVITY;

        player.rotation = Quat::from_rotation_y(yaw);
    }

    fn handle_player_horizontal_movement(
        mut query: Query<
            (&Transform, &ActionState<MovementAction>, &mut Acceleration),
            With<Player>,
        >,
    ) {
        let (transform, action_state, mut acc) = query.single_mut();

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

        direction = direction.normalize_or_zero();

        acc.0.x = direction.x * Self::ACCELERATION;
        acc.0.z = direction.z * Self::ACCELERATION;
    }

    fn handle_player_vertical_movement(
        mut commands: Commands,
        mut query: Query<
            (
                Entity,
                &Transform,
                &ActionState<MovementAction>,
                &mut Acceleration,
                Option<&Flying>,
            ),
            With<Player>,
        >,
        mut events: EventReader<CollisionEvent>,
        time: Res<Time>,
        mut prev_up: Local<Stopwatch>,
    ) {
        prev_up.tick(time.delta());
        let (entity, transform, action_state, mut acc, flying) = query.single_mut();

        for ev in events.read() {
            if ev.entity == entity && ev.y.is_some() {
                prev_up.set_elapsed(Self::DOUBLE_TAP_DELAY);
            }
        }

        let mut direction = Vec3::ZERO;

        if action_state.pressed(&MovementAction::Up) {
            direction += *transform.up();
        }
        if action_state.pressed(&MovementAction::Down) {
            direction += *transform.down();
        }

        if flying.is_some() {
            direction = direction.normalize_or_zero();
            acc.0.y = direction.y * Self::ACCELERATION;
        }

        if action_state.just_pressed(&MovementAction::Up) {
            if prev_up.elapsed() < Self::DOUBLE_TAP_DELAY {
                match flying {
                    Some(_) => {
                        commands.entity(entity).remove::<Flying>();
                        acc.0.y = 0.0;
                    }
                    None => {
                        commands.entity(entity).insert(Flying);
                    }
                };
                prev_up.set_elapsed(Self::DOUBLE_TAP_DELAY);
            } else {
                prev_up.reset();
            }
        }
    }

    fn handle_player_jumping(
        mut query: Query<
            (
                &ActionState<MovementAction>,
                &mut Velocity,
                Option<&Grounded>,
            ),
            With<Player>,
        >,
    ) {
        let (action_state, mut vel, grounded) = query.single_mut();

        if action_state.pressed(&MovementAction::Up) && grounded.is_some() {
            vel.0.y = Self::JUMP_VELOCITY;
        }
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
