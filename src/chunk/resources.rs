use super::{Chunk, Dirty};
use crate::{
    block::{Blocks, VisibleChunksIterator},
    direction::Direction,
    position::Offset,
    settings::WORLD_WIDTH,
    texture::ChunkTexture,
};
use bevy::{prelude::*, utils::HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Resource, Debug)]
pub(crate) struct Chunks {
    pub(crate) entities: HashMap<Offset, Entity>,
    pub(crate) blocks: Arc<RwLock<Blocks>>,
}

impl Chunks {
    pub(super) fn for_each_neighbor(&self, offset: Offset, mut f: impl FnMut(Entity)) {
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
}

impl FromWorld for Chunks {
    fn from_world(world: &mut World) -> Self {
        let center_offset = *world.resource::<CenterOffset>();
        let material = world.resource::<ChunkTexture>().material.clone();

        let mut entities = HashMap::with_capacity((WORLD_WIDTH * WORLD_WIDTH) as usize);

        VisibleChunksIterator::new(center_offset).for_each(|offset| {
            entities.insert(
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
            );
        });

        Self {
            entities,
            blocks: RwLock::new(Blocks::new(center_offset)).into(),
        }
    }
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct CenterOffset(pub(crate) Offset);
