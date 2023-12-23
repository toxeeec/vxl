use crate::chunk::CenterOffset;
use crate::direction::Direction;
use crate::position::{LocalPosition, Offset};
use crate::settings::{CHUNK_VOLUME, CHUNK_WIDTH, RENDER_DISTANCE, WORLD_WIDTH};
use bevy::math::IVec2;
use bevy::utils::HashMap;
use noise::utils::PlaneMapBuilder;
use noise::{utils::NoiseMapBuilder, Fbm, Perlin};
use std::cmp::Ordering;
use std::sync::Arc;

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

#[derive(Debug)]
pub(crate) struct Blocks {
    map: HashMap<Offset, Arc<[BlockId]>>,
    pub(crate) perlin: Arc<Fbm<Perlin>>,
}

#[derive(Debug)]
pub(crate) struct NeighboringChunks {
    offset: Offset,
    center: Option<Arc<[BlockId]>>,
    neighbors: [Option<Arc<[BlockId]>>; 4],
}

#[derive(Debug)]
pub(crate) struct VisibleChunksIterator {
    center_offset: Offset,
    counter: i32,
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

impl Blocks {
    pub(crate) fn new(perlin: Arc<Fbm<Perlin>>) -> Self {
        Self {
            map: HashMap::with_capacity((WORLD_WIDTH * WORLD_WIDTH) as usize),
            perlin,
        }
    }

    pub(crate) fn get_chunk(&self, offset: Offset) -> Option<Arc<[BlockId]>> {
        self.map.get(&offset).cloned()
    }

    pub(crate) fn insert_chunk(&mut self, offset: Offset, chunk: Arc<[BlockId]>) {
        self.map.insert(offset, chunk);
    }

    pub(crate) fn remove_chunk(&mut self, offset: Offset) {
        self.map.remove(&offset);
    }

    pub(crate) fn get_neighboring_chunks(&self, offset: Offset) -> NeighboringChunks {
        NeighboringChunks {
            offset,
            center: self.map.get(&offset).cloned(),
            neighbors: [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ]
            .map(|dir| self.map.get(&(offset + Offset::from(dir))).cloned()),
        }
    }
}

impl NeighboringChunks {
    #[rustfmt::skip]
    pub(crate) fn get_chunk(&self, offset: Offset) -> Option<Arc<[BlockId]>> {
        match self.offset - offset {
            Offset(IVec2 { x:  0, y:  0 }) => self.center.clone(),
            Offset(IVec2 { x:  0, y:  1 }) => self.neighbors[0].clone(),
            Offset(IVec2 { x: -1, y:  0 }) => self.neighbors[1].clone(),
            Offset(IVec2 { x:  0, y: -1 }) => self.neighbors[2].clone(),
            Offset(IVec2 { x:  1, y:  0 }) => self.neighbors[3].clone(),
            _ => panic!(),
        }
    }
}

impl VisibleChunksIterator {
    pub(crate) fn new(center_offset: CenterOffset) -> Self {
        Self {
            counter: 0,
            center_offset: center_offset.0,
        }
    }
}

impl Iterator for VisibleChunksIterator {
    type Item = Offset;

    fn next(&mut self) -> Option<Self::Item> {
        const UPPER: i32 = WORLD_WIDTH * WORLD_WIDTH - 1;
        match self.counter {
            counter @ 0..=UPPER => {
                self.counter += 1;
                Some(Offset::new(
                    (counter % WORLD_WIDTH) + self.center_offset.0.x - RENDER_DISTANCE,
                    (counter / WORLD_WIDTH) + self.center_offset.0.y - RENDER_DISTANCE,
                ))
            }
            _ => None,
        }
    }
}

pub(crate) fn generate_chunk(perlin: &Fbm<Perlin>, offset: Offset) -> Arc<[BlockId]> {
    let mut buf = vec![BlockId::Air; CHUNK_VOLUME];
    debug_assert!(buf.len() == CHUNK_VOLUME);

    let heightmap = PlaneMapBuilder::<_, 2>::new(&perlin)
        .set_size(CHUNK_WIDTH as usize, CHUNK_WIDTH as usize)
        .set_x_bounds(
            (offset.0.x * CHUNK_WIDTH) as f64,
            ((offset.0.x + 1) * CHUNK_WIDTH) as f64,
        )
        .set_y_bounds(
            (offset.0.y * CHUNK_WIDTH) as f64,
            ((offset.0.y + 1) * CHUNK_WIDTH) as f64,
        )
        .build();

    for (i, block) in buf.iter_mut().enumerate() {
        let local_pos = LocalPosition::from_index(i);

        let threshold = heightmap
            .get_value(local_pos.0.x as usize, local_pos.0.z as usize)
            .mul_add(32.0, 64.0)
            .round() as i32;
        *block = match local_pos.0.y.cmp(&threshold) {
            Ordering::Less => BlockId::Dirt,
            Ordering::Equal => BlockId::Grass,
            Ordering::Greater => BlockId::Air,
        }
    }

    buf.into()
}
