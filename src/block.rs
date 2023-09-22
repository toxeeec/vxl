use crate::chunk::{CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH};
use bevy::prelude::*;

#[derive(Bundle)]
pub(crate) struct BlockBundle {
    transform: Transform,
    visibility: Visibility,
}

impl BlockBundle {
    pub(crate) fn new(pos: Vec3, is_visible: bool) -> Self {
        Self {
            transform: Transform::from_translation(pos),
            visibility: if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        }
    }
}

pub(crate) fn block_visible(
    pos: IVec3,
    blocks: &Children,
    q_block: &Query<(&Transform, &Visibility)>,
) -> bool {
    if !block_in_bounds(pos) {
        return false;
    }
    let index = pos.y as usize * CHUNK_AREA + pos.z as usize * CHUNK_WIDTH + pos.x as usize;
    let block = blocks[index];
    match q_block.get(block) {
        Ok((_, visibility)) => visibility == Visibility::Visible,
        Err(_) => false,
    }
}

pub(crate) fn block_in_bounds(pos: IVec3) -> bool {
    pos.x >= 0
        && pos.x < CHUNK_WIDTH as i32
        && pos.y >= 0
        && pos.y < CHUNK_HEIGHT as i32
        && pos.z >= 0
        && pos.z < CHUNK_WIDTH as i32
}
