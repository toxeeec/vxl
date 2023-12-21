use super::{LocalPosition, Offset};
use crate::settings::CHUNK_WIDTH;
use bevy::prelude::*;
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub(crate) struct GlobalPosition(pub(crate) IVec3);

impl GlobalPosition {
    pub(crate) fn from_local(pos: LocalPosition, offset: Offset) -> Self {
        GlobalPosition(pos.0) + GlobalPosition::from(offset)
    }
}

impl From<Offset> for GlobalPosition {
    fn from(value: Offset) -> Self {
        Self(IVec3::new(
            value.0.x * CHUNK_WIDTH,
            0,
            value.0.y * CHUNK_WIDTH,
        ))
    }
}

impl Add<Self> for GlobalPosition {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
