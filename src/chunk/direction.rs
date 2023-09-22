use bevy::prelude::IVec3;
use strum::EnumIter;

#[derive(EnumIter, PartialEq, Clone, Copy)]
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
