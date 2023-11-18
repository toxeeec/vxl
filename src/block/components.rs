use crate::direction::Direction;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Block {
    id: BlockId,
    pub(crate) pos: IVec3,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BlockId {
    Air,
    Grass,
    Dirt,
}

impl Block {
    pub(super) fn new(id: BlockId, pos: IVec3) -> Self {
        Self { id, pos }
    }

    pub(crate) fn texture_id(self, dir: Direction) -> usize {
        match self.id {
            BlockId::Air => unreachable!(),
            BlockId::Grass => match dir {
                Direction::Up => 0,
                Direction::Down => 2,
                _ => 1,
            },
            BlockId::Dirt => 2,
        }
    }
}
