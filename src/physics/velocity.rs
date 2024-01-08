use std::ops::{AddAssign, Mul, SubAssign};

use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Default, Debug)]
pub(crate) struct LinearVelocity(Vec3);

pub(super) fn update_transforms(
    time: Res<Time>,
    mut query: Query<(&LinearVelocity, &mut Transform)>,
) {
    for (velocity, mut transform) in &mut query {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

impl LinearVelocity {
    pub const ZERO: Self = Self(Vec3::splat(0.0));

    pub(crate) fn normalize_or_zero(self) -> Self {
        Self(self.0.normalize_or_zero())
    }
}

impl AddAssign<Vec3> for LinearVelocity {
    fn add_assign(&mut self, rhs: Vec3) {
        self.0 += rhs
    }
}

impl SubAssign<Vec3> for LinearVelocity {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.0 -= rhs
    }
}

impl Mul<f32> for LinearVelocity {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}
