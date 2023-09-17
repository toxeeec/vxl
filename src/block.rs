use bevy::prelude::{Bundle, Transform, Vec3};

#[derive(Bundle)]
pub(crate) struct BlockBundle {
    transform: Transform,
}

impl BlockBundle {
    pub(crate) fn new(pos: Vec3) -> Self {
        Self {
            transform: Transform::from_translation(pos),
        }
    }
}
