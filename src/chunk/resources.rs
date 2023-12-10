use super::{Chunk, Dirty};
use crate::{
    block::{generate_blocks, BlockId},
    direction::Direction,
    position::{GlobalPosition, LocalPosition, Offset},
    settings::{CHUNK_VOLUME, RENDER_DISTANCE, WORLD_WIDTH},
    texture::ChunkTexture,
};
use bevy::{prelude::*, utils::HashMap};
use std::{mem, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub(crate) struct Blocks(Vec<Box<[BlockId]>>);

impl Blocks {
    pub(crate) fn get_chunk(&self, offset: Offset, center_offset: Offset) -> Option<&[BlockId]> {
        self.0
            .get(offset.to_index(center_offset))
            .map(|chunk| chunk.as_ref())
    }

    pub(crate) fn chunk_mut(&mut self, offset: Offset, center_offset: Offset) -> &mut [BlockId] {
        &mut self.0[offset.to_index(center_offset)]
    }

    pub(crate) fn get(&self, pos: GlobalPosition, center_offset: Offset) -> Option<BlockId> {
        let chunk_index = Offset::from(pos).to_index(center_offset);
        let local_pos = LocalPosition::from(pos);
        self.0.get(chunk_index)?.get(local_pos.to_index()).copied()
    }
}

#[derive(Resource, Debug)]
pub(crate) struct Chunks {
    pub(crate) entities: HashMap<Offset, Entity>,
    pub(crate) blocks: Arc<RwLock<Blocks>>,
    blocks_back: Blocks,
}

impl Chunks {
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

    pub(crate) fn reorder(
        &mut self,
        chunk_offsets: impl Iterator<Item = Offset>,
        center_offset: &CenterOffset,
    ) {
        let mut blocks = self.blocks.blocking_write();
        let curr_center = *center_offset.curr.blocking_read();
        for offset in chunk_offsets {
            let prev_idx = offset.to_index(center_offset.prev);
            let curr_idx = offset.to_index(curr_center);
            if let Some(prev) = blocks.0.get_mut(prev_idx) {
                if let Some(curr) = self.blocks_back.0.get_mut(curr_idx) {
                    mem::swap(prev, curr);
                }
            }
        }
        mem::swap(&mut self.blocks_back, &mut blocks);
    }
}

impl FromWorld for Chunks {
    fn from_world(world: &mut World) -> Self {
        let center_offset = *world.resource::<CenterOffset>().curr.blocking_read();
        let material = world.resource::<ChunkTexture>().material.clone();

        let entities = (0..WORLD_WIDTH * WORLD_WIDTH)
            .map(|i| {
                let offset = Offset::new(
                    (i % WORLD_WIDTH) + center_offset.0.x - RENDER_DISTANCE,
                    (i / WORLD_WIDTH) + center_offset.0.y - RENDER_DISTANCE,
                );
                (
                    offset,
                    world
                        .spawn((
                            MaterialMeshBundle {
                                transform: offset.into(),
                                material: material.clone(),
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                            Chunk,
                            Dirty,
                        ))
                        .id(),
                )
            })
            .collect();

        let blocks = Arc::new(RwLock::new(Blocks(vec![
            generate_blocks()
                .collect::<Vec<_>>()
                .into_boxed_slice();
            (WORLD_WIDTH * WORLD_WIDTH)
                as usize
        ])));

        Self {
            entities,
            blocks,
            blocks_back: Blocks(vec![
                Box::new([BlockId::default(); CHUNK_VOLUME]);
                (WORLD_WIDTH * WORLD_WIDTH) as usize
            ]),
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(crate) struct CenterOffset {
    pub(crate) prev: Offset,
    pub(crate) curr: Arc<RwLock<Offset>>,
}

impl CenterOffset {
    pub(crate) fn update(&mut self, offset: Offset) {
        let mut curr = self.curr.blocking_write();
        self.prev = *curr;
        *curr = offset;
    }
}
