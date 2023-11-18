use super::{Block, BlockId};
use bevy::prelude::*;

#[derive(Bundle, Debug)]
pub(crate) struct BlockBundle {
    block: Block,
    visibility: Visibility,
}

impl BlockBundle {
    pub(crate) fn new(block_id: BlockId, pos: IVec3) -> Self {
        let block = Block::new(block_id, pos);
        match block_id {
            BlockId::Air => Self {
                block,
                visibility: Visibility::Hidden,
            },
            BlockId::Grass => Self {
                block,
                visibility: Visibility::Visible,
            },
            BlockId::Dirt => Self {
                block,
                visibility: Visibility::Visible,
            },
        }
    }
}
