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
impl From<Direction> for IVec2 {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North                => IVec2::new( 0, -1),
            Direction::East                 => IVec2::new( 1,  0),
            Direction::South                => IVec2::new( 0,  1),
            Direction::West                 => IVec2::new(-1,  0),
            Direction::Up | Direction::Down => IVec2::new( 0,  0),
        }
    }
}

#[rustfmt::skip]
impl TryFrom<IVec2> for Direction {
    type Error = &'static str;
    fn try_from(dir: IVec2) -> Result<Self, Self::Error> {
        match dir {
            IVec2 { x:  0, y: -1 } => Ok(Direction::North),
            IVec2 { x:  1, y:  0 } => Ok(Direction::East),
            IVec2 { x:  0, y:  1 } => Ok(Direction::South),
            IVec2 { x: -1, y:  0 } => Ok(Direction::West),
            _ => Err("Invalid direction"),
        }
    }
}

#[rustfmt::skip]
impl From<Direction> for IVec3 {
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
