use std::ops::{AddAssign, Mul};

use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub(super) struct PhysicalPosition {
    current: Vec3,
    previous: Option<Vec3>,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub(super) struct Velocity(Vec3);

#[derive(Component, Clone, Copy, PartialEq, Default, Debug)]
pub(super) struct Acceleration(Vec3);

#[derive(Event, Debug)]
pub(super) struct SetAccelerationEvent {
    entity: Entity,
    acceleration: Acceleration,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PhysicsSet;

#[derive(Bundle, Default, Debug)]
pub(super) struct MovementBundle {
    velocity: Velocity,
    acceleration: Acceleration,
}

#[derive(Debug)]
pub(super) struct PhysicsPlugin;

impl PhysicalPosition {
    pub(super) fn current(&self) -> Vec3 {
        self.current
    }

    pub(super) fn previous(&self) -> Option<Vec3> {
        self.previous
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

impl From<Vec3> for Acceleration {
    #[inline]
    fn from(acceleration: Vec3) -> Self {
        Self(acceleration)
    }
}

impl SetAccelerationEvent {
    pub(super) fn new(entity: Entity, acceleration: impl Into<Acceleration>) -> Self {
        Self {
            entity,
            acceleration: acceleration.into(),
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetAccelerationEvent>()
            .add_systems(
                FixedUpdate,
                (
                    (Self::set_accelerations, Self::remove_negligible_velocities),
                    Self::apply_accelerations,
                    Self::apply_velocities,
                    Self::apply_drag,
                )
                    .chain()
                    .in_set(PhysicsSet),
            )
            .add_systems(Update, Self::interpolate_positions);
    }
}

impl PhysicsPlugin {
    const DRAG: f32 = 0.03;

    fn set_accelerations(
        mut events: EventReader<SetAccelerationEvent>,
        mut query: Query<&mut Acceleration>,
    ) {
        for ev in events.read() {
            if let Ok(mut acc) = query.get_mut(ev.entity) {
                if ev.acceleration != *acc {
                    *acc = ev.acceleration;
                }
            }
        }
    }

    fn remove_negligible_velocities(mut query: Query<&mut Velocity>) {
        const MIN_VELOCITY: f32 = 0.003;
        for mut vel in &mut query {
            if vel.0.x.abs() < MIN_VELOCITY {
                vel.0.x = 0.0;
            }
            if vel.0.y.abs() < MIN_VELOCITY {
                vel.0.y = 0.0;
            }
            if vel.0.z.abs() < MIN_VELOCITY {
                vel.0.z = 0.0;
            }
        }
    }

    fn apply_accelerations(mut query: Query<(&mut Velocity, &Acceleration)>) {
        for (mut vel, acc) in &mut query {
            vel.0 += acc.0;
        }
    }

    fn apply_velocities(mut query: Query<(&Velocity, &mut PhysicalPosition)>, time: Res<Time>) {
        let delta_seconds = time.delta_seconds();

        for (velocity, mut position) in query.iter_mut() {
            position.previous = Some(position.current);
            position.current += *velocity * delta_seconds;
        }
    }

    pub(super) fn apply_drag(mut query: Query<&mut Velocity>) {
        for mut vel in &mut query {
            vel.0 *= 1.0 - Self::DRAG;
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
