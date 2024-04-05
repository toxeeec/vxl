use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{
    player::{Player, PlayerAction},
    settings,
};

#[derive(Debug)]
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup).add_systems(
            Update,
            (Self::tilt_camera, Self::copy_camera_transform).chain(),
        );
    }
}

impl CameraPlugin {
    const EYE_HEIGHT: f32 = 1.6;

    fn setup(mut commands: Commands) {
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
        q_action: Query<&ActionState<PlayerAction>>,
    ) {
        let mut camera = q_camera.single_mut();
        let action_state = q_action.single();

        let delta = action_state.axis_pair(&PlayerAction::Turn).unwrap().y();
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
}
