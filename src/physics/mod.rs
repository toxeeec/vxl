mod velocity;

pub(crate) use velocity::LinearVelocity;

use bevy::{
    prelude::*,
    transform::systems::{propagate_transforms, sync_simple_transforms},
};
use velocity::update_transforms;

pub(super) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                update_transforms,
                (sync_simple_transforms, propagate_transforms),
            )
                .chain(),
        );
    }
}
