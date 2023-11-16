pub(crate) const PLAYER_SPEED: f32 = 10.0;
pub(crate) const SENSITIVITY: f32 = 0.1;

pub(crate) const CHUNK_WIDTH: usize = 16;
pub(crate) const CHUNK_HEIGHT: usize = 256;
pub(crate) const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub(crate) const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;

pub(crate) const RENDER_DISTANCE: usize = 1;
pub(crate) const WORLD_WIDTH: usize = RENDER_DISTANCE * 2 + 1;
