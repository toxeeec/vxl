use super::Block;
use bevy::prelude::*;

#[derive(Bundle, Debug)]
pub(crate) struct BlockBundle {
    block: Block,
    transform: Transform,
    visibility: Visibility,
}

impl BlockBundle {
    pub(crate) fn new(block: Block, pos: Vec3, is_visible: bool) -> Self {
        Self {
            block,
            transform: Transform::from_translation(pos),
            visibility: if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        }
    }
}
