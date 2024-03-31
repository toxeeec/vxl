use bevy::{diagnostic::*, prelude::*};

#[derive(Component, Debug)]
struct DiagnosticsText;

#[derive(Debug)]
pub(super) struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, Self::setup)
            .add_systems(Update, Self::display_diagnostics);
    }
}

impl DiagnosticsPlugin {
    fn setup(mut commands: Commands) {
        let text_style = TextStyle {
            font_size: 24.0,
            ..Default::default()
        };

        commands.spawn((
            TextBundle::from_sections([
                TextSection::new("FPS: ", text_style.clone()),
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

        text.sections[1].value = format!("{fps:.0}\n");
    }
}
