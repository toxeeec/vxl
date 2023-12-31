pub(crate) const PLAYER_SPEED: f32 = 100.0;
pub(crate) const SENSITIVITY: f32 = 0.1;

pub(crate) const CHUNK_WIDTH: i32 = 16;
pub(crate) const CHUNK_HEIGHT: i32 = 256;
pub(crate) const CHUNK_AREA: i32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub(crate) const CHUNK_VOLUME: usize = (CHUNK_AREA * CHUNK_HEIGHT) as usize;

pub(crate) const RENDER_DISTANCE: i32 = 20;
pub(crate) const WORLD_WIDTH: i32 = RENDER_DISTANCE * 2 + 1;
