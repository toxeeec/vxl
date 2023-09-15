use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
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
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1. })),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            unlit: true,
            ..Default::default()
        }),
        ..Default::default()
    });
}
