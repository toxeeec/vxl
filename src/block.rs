use crate::direction::Direction;
use crate::offset::Offset;
use crate::settings::{CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH};
use bevy::math::IVec3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Block {
    id: BlockId,
    pub(crate) pos: IVec3,
    pub(crate) transparency: Transparency,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            id: BlockId::Air,
            transparency: Transparency::Invisible,
            pos: IVec3::MIN,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum BlockId {
    Air,
    Grass,
    Dirt,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum Transparency {
    Invisible,
    Opaque,
}

impl Block {
    fn new(id: BlockId, pos: IVec3) -> Self {
        Self {
            id,
            pos,
            transparency: match id {
                BlockId::Air => Transparency::Invisible,
                BlockId::Grass => Transparency::Opaque,
                BlockId::Dirt => Transparency::Opaque,
            },
        }
    }

    pub(crate) fn texture_id(&self, dir: Direction) -> usize {
        debug_assert!(*self != Self::default());

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

    pub(crate) fn local_pos(&self) -> IVec3 {
        global_to_local_pos(self.pos)
    }
}

pub(crate) fn global_to_local_pos(pos: IVec3) -> IVec3 {
    let chunk_pos = IVec3::new(
        pos.x.div_euclid(CHUNK_WIDTH),
        0,
        pos.z.div_euclid(CHUNK_WIDTH),
    ) * CHUNK_WIDTH;

    pos - chunk_pos
}

pub(crate) fn generate_blocks(offset: Offset) -> impl Iterator<Item = Block> {
    (0..CHUNK_VOLUME as i32).map(move |i| {
        Block::new(
            match i / CHUNK_AREA {
                0..=2 => BlockId::Dirt,
                3 => BlockId::Grass,
                _ => BlockId::Air,
            },
            IVec3 {
                x: (i % CHUNK_WIDTH),
                y: (i / CHUNK_AREA),
                z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH),
            } + IVec3::from(offset),
        )
    })
}
