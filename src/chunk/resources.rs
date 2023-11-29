use std::mem;

use crate::{
    block::{generate_blocks, global_to_local_pos, Block},
    direction::Direction,
    offset::Offset,
    settings::{CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH, RENDER_DISTANCE, WORLD_WIDTH},
    texture::ChunkTexture,
};
use bevy::{prelude::*, utils::HashMap};

use super::{Chunk, Dirty};

#[derive(Resource, Debug)]
pub(crate) struct Chunks {
    pub(crate) offsets: Vec<Offset>,
    pub(crate) entities: HashMap<Offset, Entity>,
    pub(crate) blocks: Vec<Box<[Block]>>,
    blocks_back: Vec<Box<[Block]>>,
}

impl Chunks {
    pub(crate) fn get_block(&self, pos: IVec3, center_offset: Offset) -> Option<Block> {
        let chunk_offset = Offset::from(pos);
        let chunk_index = chunk_offset.as_index(center_offset);
        let local_pos = global_to_local_pos(pos);
        self.blocks
            .get(chunk_index)?
            .get((local_pos.x + local_pos.y * CHUNK_AREA + local_pos.z * CHUNK_WIDTH) as usize)
            .copied()
    }

    pub(crate) fn for_each_neighbor(&self, offset: Offset, mut f: impl FnMut(Entity)) {
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            if let Some(&e) = self.entities.get(&(offset.0 + Offset::from(dir).0)) {
                f(e)
            }
        }
    }

    pub(crate) fn remove(&mut self, offset: Offset) -> Option<Entity> {
        self.entities.remove(&offset)
    }

    pub(crate) fn reorder(
        &mut self,
        mut f: impl FnMut(&mut Vec<Box<[Block]>>, &mut Vec<Box<[Block]>>),
    ) {
        f(&mut self.blocks, &mut self.blocks_back);
        mem::swap(&mut self.blocks, &mut self.blocks_back);
    }
}

impl FromWorld for Chunks {
    fn from_world(world: &mut World) -> Self {
        let center_offset = world.resource::<CenterOffset>();
        let material = world.resource::<ChunkTexture>().material.clone();

        let offsets: Vec<_> = (0..WORLD_WIDTH * WORLD_WIDTH)
            .map(|i| {
                Offset::new(
                    (i % WORLD_WIDTH) + center_offset.curr().0.x - RENDER_DISTANCE,
                    (i / WORLD_WIDTH) + center_offset.curr().0.y - RENDER_DISTANCE,
                )
            })
            .collect();

        let blocks_front = offsets
            .iter()
            .map(|&offset| {
                generate_blocks(offset)
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect();

        let entities = offsets
            .iter()
            .map(|&offset| {
                (
                    offset,
                    world
                        .spawn((
                            MaterialMeshBundle {
                                transform: offset.into(),
                                material: material.clone(),
                                ..Default::default()
                            },
                            Chunk,
                            Dirty,
                        ))
                        .id(),
                )
            })
            .collect();

        Self {
            offsets: vec![Offset::default(); (WORLD_WIDTH * WORLD_WIDTH) as usize],
            entities,
            blocks: blocks_front,
            blocks_back: vec![
                Box::new([Block::default(); CHUNK_VOLUME]);
                (WORLD_WIDTH * WORLD_WIDTH) as usize
            ],
        }
    }
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct CenterOffset {
    prev: Offset,
    curr: Offset,
}

impl CenterOffset {
    pub(crate) fn update(&mut self, offset: Offset) {
        self.prev = self.curr;
        self.curr = offset;
    }

    pub(crate) fn prev(&self) -> Offset {
        self.prev
    }

    pub(crate) fn curr(&self) -> Offset {
        self.curr
    }
}
