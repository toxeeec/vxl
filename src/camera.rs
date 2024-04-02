use std::f32::consts::FRAC_PI_2;

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Reflect, Debug)]
enum CameraMovement {
    Turn,
}

#[derive(Debug)]
pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_systems(Startup, Self::setup)
            .add_systems(Update, Self::turn_camera);
    }
}

impl CameraPlugin {
    const SENSITIVITY: f32 = 0.1;

    fn setup(mut commands: Commands) {
        commands.spawn((
            Camera3dBundle {
                tonemapping: Tonemapping::None,
                transform: Transform::from_xyz(5.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
            InputManagerBundle::with_map(InputMap::new([(
                CameraMovement::Turn,
                DualAxis::mouse_motion(),
            )])),
        ));
    }

    fn turn_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>)>) {
        let (mut transform, action_state) = query.single_mut();

        let delta = action_state.axis_pair(&CameraMovement::Turn).unwrap().xy();
        if delta == Vec2::ZERO {
            return;
        }

        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        pitch -= delta.y.to_radians() * Self::SENSITIVITY;
        yaw -= delta.x.to_radians() * Self::SENSITIVITY;
        pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    }
}
