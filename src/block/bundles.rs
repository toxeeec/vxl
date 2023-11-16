use super::Block;
use bevy::prelude::*;

#[derive(Bundle, Debug)]
pub(crate) struct BlockBundle {
    block: Block,
    visibility: Visibility,
    transform: TransformBundle,
}

impl BlockBundle {
    pub(crate) fn new(block: Block, pos: Vec3) -> Self {
        Self {
            block,
            transform: TransformBundle::from_transform(Transform::from_translation(pos)),
            visibility: if block.is_visible() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        }
    }
}
