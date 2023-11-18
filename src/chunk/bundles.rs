use super::{Chunk, Dirty};
use bevy::prelude::*;

#[derive(Bundle, Debug)]
pub(super) struct ChunkBundle {
    transform: Transform,
    chunk: Chunk,
    dirty: Dirty,
}

impl ChunkBundle {
    pub(super) fn new(transform: Transform) -> Self {
        ChunkBundle {
            transform,
            chunk: Chunk,
            dirty: Dirty,
        }
    }
}
