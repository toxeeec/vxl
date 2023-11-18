use super::offset::index_from_offset;
use crate::{block::block_in_bounds, settings::CHUNK_WIDTH};
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub(crate) struct Chunks(Vec<Entity>);

impl Chunks {
    pub(crate) fn new(vec: Vec<Entity>) -> Self {
        Self(vec)
    }

    pub(crate) fn get_by_pos(&self, pos: IVec3) -> Option<Entity> {
        if !block_in_bounds(pos) {
            return None;
        };
        let chunk_offset = IVec2::new(pos.x.div_euclid(CHUNK_WIDTH), pos.z.div_euclid(CHUNK_WIDTH));
        let index = index_from_offset(chunk_offset);
        self.0.get(index).copied()
    }
}
