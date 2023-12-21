use crate::settings::SENSITIVITY;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub(crate) enum CameraMovement {
    Rotation,
}

fn rotate_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera3d>>) {
    let (mut transform, action_state) = query.single_mut();

    let delta = action_state
        .axis_pair(CameraMovement::Rotation)
        .unwrap()
        .xy();

    let rotation = transform.rotation;
    let (mut yaw, mut pitch, _) = rotation.to_euler(EulerRot::YXZ);

    if delta == Vec2::ZERO {
        return;
    }
    pitch -= delta.y.to_radians() * SENSITIVITY;
    yaw -= delta.x.to_radians() * SENSITIVITY;
    pitch = pitch.clamp(-89.9f32.to_radians(), 89.9f32.to_radians());

    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

#[derive(Debug)]
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_systems(Update, rotate_camera);
    }
}
