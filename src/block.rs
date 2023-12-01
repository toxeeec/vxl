use crate::direction::Direction;
use crate::settings::{CHUNK_AREA, CHUNK_VOLUME};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub(crate) enum BlockId {
    #[default]
    Air,
    Grass,
    Dirt,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum Transparency {
    Invisible,
    Opaque,
}

impl BlockId {
    pub(crate) fn transparency(self) -> Transparency {
        match self {
            BlockId::Air => Transparency::Invisible,
            BlockId::Grass => Transparency::Opaque,
            BlockId::Dirt => Transparency::Opaque,
        }
    }

    pub(crate) fn texture_id(self, dir: Direction) -> usize {
        match self {
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

pub(crate) fn generate_blocks() -> impl Iterator<Item = BlockId> {
    (0..CHUNK_VOLUME as i32).map(move |i| match i / CHUNK_AREA {
        0..=2 => BlockId::Dirt,
        3 => BlockId::Grass,
        _ => BlockId::Air,
    })
}
