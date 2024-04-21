use bevy::prelude::*;
use strum::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug)]
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
    #[inline]
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => IVec3::new( 0,  0, -1),
            Direction::East  => IVec3::new( 1,  0,  0),
            Direction::South => IVec3::new( 0,  0,  1),
            Direction::West  => IVec3::new(-1,  0,  0),
            Direction::Up    => IVec3::new( 0,  1,  0),
            Direction::Down  => IVec3::new( 0, -1,  0),
        }
    }
}
