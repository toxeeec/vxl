use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use block::create_block_mesh;
use diagnostics::DiagnosticsPlugin;

mod block;
mod diagnostics;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DiagnosticsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera3dBundle {
        tonemapping: Tonemapping::None,
        transform: Transform::from_xyz(5.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(MaterialMeshBundle {
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            unlit: true,
            ..Default::default()
        }),
        mesh: meshes.add(create_block_mesh()),
        ..Default::default()
    });
}
