mod bundles;
mod components;

pub(crate) use bundles::BlockBundle;
pub(crate) use components::{Block, BlockId};

use crate::settings::{CHUNK_AREA, CHUNK_HEIGHT, CHUNK_WIDTH, RENDER_DISTANCE};
use bevy::prelude::*;

pub(super) fn block_in_bounds(pos: IVec3) -> bool {
    pos.x >= -(RENDER_DISTANCE * CHUNK_WIDTH)
        && pos.x < ((RENDER_DISTANCE + 1) * CHUNK_WIDTH)
        && pos.y >= 0
        && pos.y < CHUNK_HEIGHT
        && pos.z >= -(RENDER_DISTANCE * CHUNK_WIDTH)
        && pos.z < ((RENDER_DISTANCE + 1) * CHUNK_WIDTH)
}

pub(crate) fn global_to_local_pos(pos: IVec3) -> IVec3 {
    let chunk_pos = IVec3::new(
        pos.x.div_euclid(CHUNK_WIDTH),
        0,
        pos.z.div_euclid(CHUNK_WIDTH),
    ) * CHUNK_WIDTH;

    pos - chunk_pos
}

pub(crate) fn pos_to_index(pos: IVec3) -> usize {
    (pos.x + pos.y * CHUNK_AREA + pos.z * CHUNK_WIDTH) as usize
}
