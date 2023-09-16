use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::*,
    window::CursorGrabMode,
};

const PLAYER_SPEED: f32 = 10.0;
const SENSITIVITY: f32 = 0.1;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, camera_rotation, fps_display))
        .run();
}

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window>,
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

    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    let text_style = TextStyle {
        font_size: 24.0,
        ..Default::default()
    };

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("FPS: ", text_style.clone()),
            TextSection::from_style(text_style),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..Default::default()
        }),
        FpsText,
    ));
}

fn camera_movement(
    mut query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut camera_transform = query.single_mut();
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

fn camera_rotation(
    mut query: Query<&mut Transform, With<Camera>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let mut camera_transform = query.single_mut();
    let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

    let delta = motion_evr.iter().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);
    pitch -= delta.y.to_radians() * SENSITIVITY;
    yaw -= delta.x.to_radians() * SENSITIVITY;
    pitch = pitch.clamp(-89.9f32.to_radians(), 89.9f32.to_radians());

    camera_transform.rotation =
        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

fn fps_display(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    let mut text = query.single_mut();
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps.smoothed() {
            text.sections[1].value = format!("{fps:.0}");
        }
    }
}
