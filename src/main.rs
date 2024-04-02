use bevy::{prelude::*, window::CursorGrabMode};
use block::create_block_mesh;
use camera::CameraPlugin;
use diagnostics::DiagnosticsPlugin;

mod block;
mod camera;
mod diagnostics;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraPlugin, DiagnosticsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut query: Query<&mut Window>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut window = query.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

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
