mod bundles;
mod components;

pub(crate) use bundles::BlockBundle;
pub(crate) use components::Block;

use crate::chunk::{CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH};
use bevy::prelude::*;

pub(super) fn block_visible(
    pos: IVec3,
    blocks: &Children,
    q_block: &Query<(&Block, &Transform, &Visibility)>,
) -> bool {
    if !block_in_bounds(pos) {
        return false;
    }
    let index = pos.y as usize * CHUNK_AREA + pos.z as usize * CHUNK_WIDTH + pos.x as usize;
    let (_, _, visibility) = q_block.get(blocks[index]).unwrap();
    visibility == Visibility::Visible
}

fn block_in_bounds(pos: IVec3) -> bool {
    pos.x >= 0
        && pos.x < CHUNK_WIDTH as i32
        && pos.y >= 0
        && pos.y < CHUNK_HEIGHT as i32
        && pos.z >= 0
        && pos.z < CHUNK_WIDTH as i32
}
