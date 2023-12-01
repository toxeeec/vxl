use crate::position::{GlobalPosition, Offset};
use bevy::prelude::*;
use strum::EnumIter;

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
impl From<Direction> for GlobalPosition {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => Self(IVec3::new( 0,  0, -1)),
            Direction::East  => Self(IVec3::new( 1,  0,  0)),
            Direction::South => Self(IVec3::new( 0,  0,  1)),
            Direction::West  => Self(IVec3::new(-1,  0,  0)),
            Direction::Up    => Self(IVec3::new( 0,  1,  0)),
            Direction::Down  => Self(IVec3::new( 0, -1,  0)),
        }
    }
}

#[rustfmt::skip]
impl From<Direction> for Offset {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => Self::new( 0,  1),
            Direction::East  => Self::new( 1,  0),
            Direction::South => Self::new( 0,  1),
            Direction::West  => Self::new(-1,  0),
            _ => unreachable!(),
        }
    }
}
