use super::{offset::transform_from_offset, Chunk, Dirty};
use bevy::prelude::*;

#[derive(Bundle, Debug)]
pub(super) struct ChunkBundle {
    transform: TransformBundle,
    chunk: Chunk,
    dirty: Dirty,
}

impl ChunkBundle {
    pub(super) fn new(offset: IVec2) -> Self {
        ChunkBundle {
            transform: TransformBundle::from_transform(transform_from_offset(offset)),
            chunk: Chunk,
            dirty: Dirty,
        }
    }
}
