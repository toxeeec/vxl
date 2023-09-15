use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

const PLAYER_SPEED: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        tonemapping: Tonemapping::None,
        transform: Transform::from_xyz(5.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            unlit: true,
            ..Default::default()
        }),
        ..Default::default()
    });
}

fn camera_movement(
    mut query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut camera_transform = query.get_single_mut().unwrap();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::W) {
        direction += camera_transform.forward();
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += camera_transform.back();
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction += camera_transform.left();
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += camera_transform.right();
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction += camera_transform.up();
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction += camera_transform.down();
    }

    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    camera_transform.translation += movement;
}
