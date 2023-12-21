use crate::chunk::CenterOffset;
use crate::direction::Direction;
use crate::position::{LocalPosition, Offset};
use crate::settings::{CHUNK_VOLUME, CHUNK_WIDTH, RENDER_DISTANCE, WORLD_WIDTH};
use bevy::utils::HashMap;
use noise::utils::PlaneMapBuilder;
use noise::*;
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
    map: HashMap<Offset, Box<[BlockId]>>,
    pool: Vec<Box<[BlockId]>>,
    perlin: Arc<Fbm<Perlin>>,
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
    pub(crate) fn new(center_offset: CenterOffset) -> Self {
        let seed = 0;
        let perlin = Fbm::<Perlin>::new(seed).set_frequency(0.005);
        Self {
            map: VisibleChunksIterator::new(center_offset)
                .map(|offset| (offset, generate_chunk(None, &perlin, offset)))
                .collect(),
            pool: Vec::with_capacity((WORLD_WIDTH * 2 - 1) as usize),
            perlin: perlin.into(),
        }
    }

    pub(crate) fn get_chunk(&self, offset: Offset) -> Option<&[BlockId]> {
        self.map.get(&offset).map(|chunk| chunk.as_ref())
    }

    pub(crate) fn remove_chunk(&mut self, offset: Offset) {
        if let Some(chunk) = self.map.remove(&offset) {
            self.pool.push(chunk);
        }
    }

    pub(crate) fn generate_chunk(&mut self, offset: Offset) {
        self.map.insert(
            offset,
            generate_chunk(self.pool.pop(), &self.perlin, offset),
        );
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

fn generate_chunk(
    buf: Option<Box<[BlockId]>>,
    perlin: &Fbm<Perlin>,
    offset: Offset,
) -> Box<[BlockId]> {
    let mut buf = buf.unwrap_or_else(|| vec![BlockId::Air; CHUNK_VOLUME].into_boxed_slice());
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

    buf
}
