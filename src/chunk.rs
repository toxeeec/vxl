use bevy::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

pub(crate) const CHUNK_WIDTH: usize = 16;
pub(crate) const CHUNK_HEIGHT: usize = 256;
pub(crate) const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub(crate) const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;

#[rustfmt::skip]
const FACES_VERTICES: [[Vec3; 4]; 6] = [
    [
        Vec3 {x: 1.0, y: 1.0, z: 0.0}, // north(-z)
        Vec3 {x: 0.0, y: 1.0, z: 0.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0}
    ],
    [
        Vec3 {x: 1.0, y: 1.0, z: 1.0}, // east(+x)
        Vec3 {x: 1.0, y: 1.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 1.0}, // south(+z)
        Vec3 {x: 1.0, y: 1.0, z: 1.0},
        Vec3 {x: 1.0, y: 0.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 0.0}, // west(-x)
        Vec3 {x: 0.0, y: 1.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 0.0}, // up(+y)
        Vec3 {x: 1.0, y: 1.0, z: 0.0},
        Vec3 {x: 1.0, y: 1.0, z: 1.0},
        Vec3 {x: 0.0, y: 1.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 0.0, z: 1.0}, // down(-y)
        Vec3 {x: 1.0, y: 0.0, z: 1.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0}
    ]
];

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

pub(crate) const VERTICES_CAPACITY: usize =
    CHUNK_VOLUME / 2 * FACES_VERTICES.len() * FACES_VERTICES[0].len();
pub(crate) const INDICES_CAPACITY: usize =
    CHUNK_VOLUME / 2 * FACES_VERTICES.len() * FACE_INDICES.len();

#[derive(Component)]
pub(crate) struct Chunk;

impl Chunk {
    pub(crate) fn mesh(
        blocks: &Children,
        q_block: &Query<(&Transform, &Visibility)>,
        positions: &mut Vec<Vec3>,
        indices: &mut Vec<u32>,
    ) {
        for &block in blocks.iter() {
            if let Ok((&Transform { translation, .. }, visibility)) = q_block.get(block) {
                if visibility == Visibility::Hidden {
                    continue;
                }
                for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                    if block_visible(translation.as_ivec3() + IVec3::from(dir), blocks, q_block) {
                        continue;
                    }
                    for index in FACE_INDICES {
                        indices.push(positions.len() as u32 + index);
                    }
                    for vertex in vertices {
                        positions.push(vertex + translation);
                    }
                }
            }
        }
    }

    fn block_in_bounds(pos: IVec3) -> bool {
        pos.x >= 0
            && pos.x < CHUNK_WIDTH as i32
            && pos.y >= 0
            && pos.y < CHUNK_HEIGHT as i32
            && pos.z >= 0
            && pos.z < CHUNK_WIDTH as i32
    }
}

fn block_visible(
    pos: IVec3,
    blocks: &Children,
    q_block: &Query<(&Transform, &Visibility)>,
) -> bool {
    if !Chunk::block_in_bounds(pos) {
        return false;
    }
    let index = pos.y as usize * CHUNK_AREA + pos.z as usize * CHUNK_WIDTH + pos.x as usize;
    let block = blocks[index];
    match q_block.get(block) {
        Ok((_, visibility)) => visibility == Visibility::Visible,
        Err(_) => false,
    }
}

#[derive(EnumIter, PartialEq, Clone, Copy)]
enum Direction {
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
