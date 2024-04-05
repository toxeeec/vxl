use bevy::{diagnostic::*, prelude::*};

use crate::player::Player;

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
            .add_systems(Startup, Self::setup)
            .add_systems(
                Update,
                (Self::update_position, Self::display_diagnostics).chain(),
            );
    }
}

impl DiagnosticsPlugin {
    const POS_X: DiagnosticPath = DiagnosticPath::const_new("pos_x");
    const POS_Y: DiagnosticPath = DiagnosticPath::const_new("pos_y");
    const POS_Z: DiagnosticPath = DiagnosticPath::const_new("pos_z");

    fn setup(mut commands: Commands) {
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

    fn update_position(query: Query<&Transform, With<Player>>, mut diagnostics: Diagnostics) {
        let pos = query.single().translation;
        diagnostics.add_measurement(&Self::POS_X, || pos.x as f64);
        diagnostics.add_measurement(&Self::POS_Y, || pos.y as f64);
        diagnostics.add_measurement(&Self::POS_Z, || pos.z as f64);
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

        text.sections[1].value = format!("{fps:.0}\n");
        text.sections[3].value = format!("{pos_x:.3}/{pos_y:.3}/{pos_z:.3}\n");
    }
}
