use crate::direction::Direction;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum Block {
    Air,
    Grass,
    Dirt,
}

impl Block {
    pub(crate) fn texture_id(self, dir: Direction) -> usize {
        match self {
            Block::Air => unreachable!(),
            Block::Grass => match dir {
                Direction::Up => 0,
                Direction::Down => 2,
                _ => 1,
            },
            Block::Dirt => 2,
        }
    }

    pub(super) fn is_visible(self) -> bool {
        match self {
            Block::Air => false,
            Block::Grass => true,
            Block::Dirt => true,
        }
    }
}
