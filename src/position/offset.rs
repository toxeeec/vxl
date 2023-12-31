use super::GlobalPosition;
use crate::settings::{CHUNK_WIDTH, RENDER_DISTANCE};
use bevy::prelude::*;
use std::ops::{Add, Sub};

#[derive(PartialEq, Eq, Clone, Copy, Default, Hash, Debug)]
pub(crate) struct Offset(pub(crate) IVec2);

impl Offset {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Offset(IVec2::new(x, y))
    }

    pub(crate) fn in_bounds(self, center_offset: Offset) -> bool {
        let dist = (self.0 - center_offset.0).abs();
        dist.x <= RENDER_DISTANCE && dist.y <= RENDER_DISTANCE
    }
}

impl From<IVec3> for Offset {
    fn from(value: IVec3) -> Self {
        Self::new(
            value.x.div_euclid(CHUNK_WIDTH),
            value.z.div_euclid(CHUNK_WIDTH),
        )
    }
}

impl From<GlobalPosition> for Offset {
    fn from(value: GlobalPosition) -> Self {
        value.0.into()
    }
}

impl From<Transform> for Offset {
    fn from(value: Transform) -> Self {
        value.translation.as_ivec3().into()
    }
}

impl From<&Transform> for Offset {
    fn from(value: &Transform) -> Self {
        (*value).into()
    }
}

impl From<Offset> for IVec3 {
    fn from(value: Offset) -> Self {
        IVec3::new(value.0.x * CHUNK_WIDTH, 0, value.0.y * CHUNK_WIDTH)
    }
}

impl From<Offset> for Transform {
    fn from(value: Offset) -> Self {
        Transform::from_xyz(
            value.0.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            value.0.y as f32 * CHUNK_WIDTH as f32,
        )
    }
}

impl Add<Self> for Offset {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Self> for Offset {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
