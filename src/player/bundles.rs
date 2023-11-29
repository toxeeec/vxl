use super::Player;
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    camera: Camera3dBundle,
    player: Player,
}

impl PlayerBundle {
    pub(crate) fn new(transform: Transform) -> Self {
        Self {
            camera: Camera3dBundle {
                tonemapping: Tonemapping::None,
                transform,
                ..Default::default()
            },
            player: Player,
        }
    }
}
