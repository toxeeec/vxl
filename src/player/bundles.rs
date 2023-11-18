use super::Player;
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    camera: Camera3dBundle,
    player: Player,
}

impl PlayerBundle {
    pub(crate) fn new(pos: Vec3) -> Self {
        Self {
            camera: Camera3dBundle {
                tonemapping: Tonemapping::None,
                transform: Transform::from_translation(pos).looking_at(Vec3::splat(8.0), Vec3::Y),
                ..Default::default()
            },
            player: Player,
        }
    }
}
