use crate::settings::{CHUNK_WIDTH, RENDER_DISTANCE, WORLD_WIDTH};
use bevy::prelude::*;

pub(super) fn transform_from_offset(offset: IVec2) -> Transform {
    let x = offset.x as f32 * CHUNK_WIDTH as f32;
    let z = offset.y as f32 * CHUNK_WIDTH as f32;
    Transform::from_xyz(x, 0.0, z)
}

pub(super) fn index_from_offset(offset: IVec2) -> usize {
    ((offset.y + RENDER_DISTANCE) * WORLD_WIDTH + offset.x + RENDER_DISTANCE) as usize
}

pub(super) fn visible_chunks_offsets() -> impl Iterator<Item = IVec2> {
    let size = RENDER_DISTANCE * 2 + 1;

    (0..size * size)
        .map(move |i| IVec2::new((i % size) - RENDER_DISTANCE, (i / size) - RENDER_DISTANCE))
}
