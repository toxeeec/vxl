use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct LoadingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GameplaySet;
