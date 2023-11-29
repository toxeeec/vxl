use bevy::prelude::*;
use strum::EnumIter;

use crate::offset::Offset;

#[derive(EnumIter, PartialEq, Clone, Copy, Debug)]
pub(super) enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

#[rustfmt::skip]
impl From<Direction> for IVec3 {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => Self {x:  0, y:  0, z: -1},
            Direction::East  => Self {x:  1, y:  0, z:  0},
            Direction::South => Self {x:  0, y:  0, z:  1},
            Direction::West  => Self {x: -1, y:  0, z:  0},
            Direction::Up    => Self {x:  0, y:  1, z:  0},
            Direction::Down  => Self {x:  0, y: -1, z:  0},
        }
    }
}

impl From<Direction> for Offset {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => Self::new(0, 1),
            Direction::East => Self::new(1, 0),
            Direction::South => Self::new(0, 1),
            Direction::West => Self::new(-1, 0),
            _ => unreachable!(),
        }
    }
}
