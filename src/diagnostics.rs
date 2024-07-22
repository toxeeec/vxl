use bevy::{diagnostic::*, prelude::*};
use noise::NoiseFn;

use crate::{
    physics::{PhysicalPosition, Velocity},
    player::Player,
    sets::GameplaySet,
    state::AppState,
    world::{Noise, WorldgenParams},
};

#[derive(Component, Debug)]
struct DiagnosticsText;

#[derive(Debug)]
pub(super) struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .register_diagnostic(Diagnostic::new(Self::POS_X))
            .register_diagnostic(Diagnostic::new(Self::POS_Y))
            .register_diagnostic(Diagnostic::new(Self::POS_Z))
            .register_diagnostic(Diagnostic::new(Self::BLOCKS_PER_SECOND))
            .register_diagnostic(Diagnostic::new(Self::HILLINESS))
            .add_systems(OnEnter(AppState::InGame), Self::spawn_diagnostics_text)
            .add_systems(
                FixedUpdate,
                (
                    Self::update_position,
                    Self::update_blocks_per_second,
                    Self::update_hilliness.run_if(resource_exists::<WorldgenParams>),
                )
                    .in_set(GameplaySet),
            )
            .add_systems(Update, (Self::display_diagnostics).in_set(GameplaySet));
    }
}

impl DiagnosticsPlugin {
    const POS_X: DiagnosticPath = DiagnosticPath::const_new("pos_x");
    const POS_Y: DiagnosticPath = DiagnosticPath::const_new("pos_y");
    const POS_Z: DiagnosticPath = DiagnosticPath::const_new("pos_z");
    const BLOCKS_PER_SECOND: DiagnosticPath = DiagnosticPath::const_new("blocks_per_second");
    const HILLINESS: DiagnosticPath = DiagnosticPath::const_new("hilliness");

    fn spawn_diagnostics_text(mut commands: Commands) {
        let text_style = TextStyle {
            font_size: 24.0,
            ..Default::default()
        };

        commands.spawn((
            TextBundle::from_sections([
                TextSection::new("FPS: ", text_style.clone()),
                TextSection::from_style(text_style.clone()),
                TextSection::new("X/Y/Z: ", text_style.clone()),
                TextSection::from_style(text_style.clone()),
                TextSection::new("B/s: ", text_style.clone()),
                TextSection::from_style(text_style.clone()),
                TextSection::new("Hilliness: ", text_style.clone()),
                TextSection::from_style(text_style.clone()),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            }),
            DiagnosticsText,
        ));
    }

    fn update_position(
        query: Query<&PhysicalPosition, With<Player>>,
        mut diagnostics: Diagnostics,
    ) {
        let pos = query.single().current();
        diagnostics.add_measurement(&Self::POS_X, || pos.x as f64);
        diagnostics.add_measurement(&Self::POS_Y, || pos.y as f64);
        diagnostics.add_measurement(&Self::POS_Z, || pos.z as f64);
    }

    fn update_blocks_per_second(
        query: Query<&Velocity, With<Player>>,
        mut diagnostics: Diagnostics,
    ) {
        let vel = query.single();
        diagnostics.add_measurement(&Self::BLOCKS_PER_SECOND, || vel.magnitude() as f64);
    }

    fn update_hilliness(
        query: Query<&PhysicalPosition, With<Player>>,
        noise: Res<Noise>,
        mut diagnostics: Diagnostics,
    ) {
        let pos = query.single().current().xz().round().as_dvec2().to_array();
        let hilliness = (noise.hilliness().get(pos) + 1.0) / 2.0;
        diagnostics.add_measurement(&Self::HILLINESS, || hilliness);
    }

    fn display_diagnostics(
        mut query: Query<&mut Text, With<DiagnosticsText>>,
        diagnostics: Res<DiagnosticsStore>,
    ) {
        let mut text = query.single_mut();

        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .unwrap()
            .smoothed()
            .unwrap_or_default();

        let pos_x = diagnostics
            .get(&Self::POS_X)
            .unwrap()
            .value()
            .unwrap_or_default();

        let pos_y = diagnostics
            .get(&Self::POS_Y)
            .unwrap()
            .value()
            .unwrap_or_default();

        let pos_z = diagnostics
            .get(&Self::POS_Z)
            .unwrap()
            .value()
            .unwrap_or_default();

        let bps = diagnostics
            .get(&Self::BLOCKS_PER_SECOND)
            .unwrap()
            .value()
            .unwrap_or_default();

        let hilliness = diagnostics
            .get(&Self::HILLINESS)
            .unwrap()
            .value()
            .unwrap_or_default();

        text.sections[1].value = format!("{fps:.0}\n");
        text.sections[3].value = format!("{pos_x:.3}/{pos_y:.3}/{pos_z:.3}\n");
        text.sections[5].value = format!("{bps:.4}\n");
        text.sections[7].value = format!("{hilliness:.4}\n");
    }
}
