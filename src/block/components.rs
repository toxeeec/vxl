use crate::direction::Direction;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum Block {
    Grass,
    Dirt,
}

impl Block {
    pub(crate) fn texture_id(self, dir: Direction) -> usize {
        match self {
            Block::Grass => match dir {
                Direction::Up => 0,
                Direction::Down => 2,
                _ => 1,
            },
            Block::Dirt => 2,
        }
    }
}
