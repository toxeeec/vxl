use std::ops::{AddAssign, Mul};

use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub(super) struct PhysicalPosition {
    current: Vec3,
    previous: Option<Vec3>,
}

#[derive(Component, Clone, Copy, PartialEq, Default, Debug)]
pub(super) struct Velocity(Vec3);

#[derive(Debug)]
pub(super) struct PhysicsPlugin;

impl PhysicalPosition {
    pub(super) fn current(&self) -> Vec3 {
        self.current
    }
}

impl From<Transform> for PhysicalPosition {
    #[inline]
    fn from(transform: Transform) -> Self {
        Self {
            current: transform.translation,
            previous: None,
        }
    }
}

impl Velocity {
    pub(super) fn new(velocity: Vec3) -> Self {
        Self(velocity)
    }

    pub(super) fn magnitude(self) -> f32 {
        self.0.length()
    }
}

impl Mul<f32> for Velocity {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl AddAssign<Velocity> for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Velocity) {
        *self += rhs.0
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, Self::apply_velocities)
            .add_systems(Update, Self::interpolate_positions);
    }
}

impl PhysicsPlugin {
    fn apply_velocities(mut query: Query<(&Velocity, &mut PhysicalPosition)>, time: Res<Time>) {
        let delta_seconds = time.delta_seconds();

        for (velocity, mut position) in query.iter_mut() {
            position.previous = Some(position.current);
            position.current += *velocity * delta_seconds;
        }
    }

    fn interpolate_positions(
        mut query: Query<(&PhysicalPosition, &mut Transform)>,
        time: Res<Time<Fixed>>,
    ) {
        let delta_seconds = time.delta_seconds();
        let overstep = time.overstep().as_secs_f32();

        for (position, mut transform) in query.iter_mut() {
            match position.previous {
                Some(prev) => {
                    transform.translation = prev.lerp(position.current, overstep / delta_seconds)
                }
                None => transform.translation = position.current,
            }
        }
    }
}
