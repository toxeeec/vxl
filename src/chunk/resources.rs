use std::mem;

use crate::{
    block::{generate_blocks, BlockId},
    direction::Direction,
    position::{GlobalPosition, LocalPosition, Offset},
    settings::{CHUNK_VOLUME, RENDER_DISTANCE, WORLD_WIDTH},
    texture::ChunkTexture,
};
use bevy::{prelude::*, utils::HashMap};

use super::{Chunk, Dirty};

#[derive(Resource, Debug)]
pub(crate) struct Chunks {
    pub(crate) offsets: Vec<Offset>,
    pub(crate) entities: HashMap<Offset, Entity>,
    pub(crate) blocks: Vec<Box<[BlockId]>>,
    blocks_back: Vec<Box<[BlockId]>>,
}

impl Chunks {
    pub(crate) fn get_block(&self, pos: GlobalPosition, center_offset: Offset) -> Option<BlockId> {
        let chunk_index = Offset::from(pos).to_index(center_offset);
        let local_pos = LocalPosition::from(pos);
        self.blocks
            .get(chunk_index)?
            .get(local_pos.to_index())
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
        chunk_offsets: impl Iterator<Item = Offset>,
        center_offset: CenterOffset,
    ) {
        for offset in chunk_offsets {
            let prev_idx = offset.to_index(center_offset.prev());
            let curr_idx = offset.to_index(center_offset.curr());
            mem::swap(&mut self.blocks[prev_idx], &mut self.blocks_back[curr_idx]);
        }
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
            .map(|_| generate_blocks().collect::<Vec<_>>().into_boxed_slice())
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
                Box::new([BlockId::default(); CHUNK_VOLUME]);
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
