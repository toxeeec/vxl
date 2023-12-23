use super::global_position::GlobalPosition;
use crate::settings::{CHUNK_AREA, CHUNK_WIDTH};
use bevy::prelude::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct LocalPosition(pub(crate) IVec3);

impl LocalPosition {
    pub(crate) fn from_index(i: usize) -> Self {
        let i = i as i32;
        LocalPosition(IVec3 {
            x: (i % CHUNK_WIDTH),
            y: (i / CHUNK_AREA),
            z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH),
        })
    }

    pub(crate) fn to_index(self) -> usize {
        (self.0.x + self.0.y * CHUNK_AREA + self.0.z * CHUNK_WIDTH) as usize
    }
}

impl From<LocalPosition> for IVec3 {
    fn from(value: LocalPosition) -> Self {
        value.0
    }
}

impl From<GlobalPosition> for LocalPosition {
    fn from(pos: GlobalPosition) -> Self {
        let chunk_pos = IVec3::new(
            pos.0.x.div_euclid(CHUNK_WIDTH),
            0,
            pos.0.z.div_euclid(CHUNK_WIDTH),
        ) * CHUNK_WIDTH;

        Self(pos.0 - chunk_pos)
    }
}
