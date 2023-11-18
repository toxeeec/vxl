mod bundles;
mod systems;

use bevy::prelude::*;
use systems::{player_movement, player_rotation, spawn_player};

#[derive(Component, Debug)]
struct Player;

#[derive(Debug)]
pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, player_rotation));
    }
}
