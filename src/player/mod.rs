mod bundles;
mod systems;

pub(crate) use systems::player_movement;

use crate::chunk::CenterOffset;
use bevy::prelude::*;
use systems::{player_rotation, spawn_player};

#[derive(Component, Debug)]
pub(crate) struct Player;

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CenterOffset>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_rotation));
    }
}
