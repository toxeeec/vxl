use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum AppState {
    #[default]
    Loading,
    Generating,
    InGame,
}
