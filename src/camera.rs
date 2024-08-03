use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{
    physics::{PhysicsSet, Sprinting},
    player::{CameraAction, Player},
    sets::GameplaySet,
    settings::{self, FOV},
    state::AppState,
};

#[derive(Debug)]
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), Self::spawn_camera)
            .add_systems(
                FixedUpdate,
                (Self::update_fov).after(PhysicsSet).in_set(GameplaySet),
            )
            .add_systems(
                Update,
                (Self::tilt_camera, Self::copy_camera_transform)
                    .chain()
                    .after(PhysicsSet)
                    .in_set(GameplaySet),
            );
    }
}

impl CameraPlugin {
    const EYE_HEIGHT: f32 = 1.6;

    fn spawn_camera(mut commands: Commands) {
        commands.spawn(Camera3dBundle {
            projection: PerspectiveProjection {
                fov: settings::FOV,
                ..Default::default()
            }
            .into(),
            tonemapping: Tonemapping::None,
            ..Default::default()
        });
    }

    fn tilt_camera(
        mut q_camera: Query<&mut Transform, With<Camera>>,
        q_action: Query<&ActionState<CameraAction>>,
    ) {
        let mut camera = q_camera.single_mut();
        let action_state = q_action.single();

        let delta = action_state
            .axis_pair(&CameraAction::Turn)
            .unwrap_or_default()
            .y();
        if delta == 0.0 {
            return;
        }

        let (_, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
        pitch -= delta.to_radians() * settings::SENSITIVITY;
        pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        camera.rotation = Quat::from_rotation_x(pitch);
    }

    fn copy_camera_transform(
        mut set: ParamSet<(
            Query<&Transform, With<Player>>,
            Query<&mut Transform, With<Camera>>,
        )>,
    ) {
        let q_player = set.p0();
        let player = q_player.single();
        let player_translation = player.translation;
        let player_rotation = player.rotation;

        let mut q_camera = set.p1();
        let mut camera = q_camera.single_mut();
        camera.translation = player_translation + Vec3::new(0.0, Self::EYE_HEIGHT, 0.0);

        let (yaw, _, _) = player_rotation.to_euler(EulerRot::YXZ);
        let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);

        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    pub(super) fn update_fov(
        mut q_proj: Query<&mut Projection>,
        q_sprinting: Query<Option<&Sprinting>, With<Player>>,
        time: Res<Time>,
    ) {
        const SPRINTING_FOV: f32 = FOV * 1.1;
        const FOV_CHANGE_DURATION: f32 = 0.15;

        let mut projection = q_proj.single_mut();
        if let Projection::Perspective(PerspectiveProjection { fov, .. }) = &mut *projection {
            let is_sprinting = q_sprinting.single().is_some();
            let delta_seconds = time.delta_seconds();

            let (base_fov, target_fov) = if is_sprinting {
                (FOV, SPRINTING_FOV)
            } else {
                (SPRINTING_FOV, FOV)
            };

            let fov_diff = target_fov - base_fov;
            let fov_step = fov_diff * delta_seconds / FOV_CHANGE_DURATION;
            *fov = (*fov + fov_step).clamp(FOV, SPRINTING_FOV);
        }
    }
}
