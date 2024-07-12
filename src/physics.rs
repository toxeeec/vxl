use std::ops::{AddAssign, Mul};

use bevy::{math::bounding::Bounded3d, prelude::*};

use crate::world::Chunks;

#[derive(Component, Default, Debug)]
pub(super) struct PhysicalPosition {
    current: Vec3,
    previous: Option<Vec3>,
}

#[derive(Component, Default, Debug)]
pub(super) struct RigidBody(Cuboid);

#[derive(Component, Clone, Copy, Default, Debug)]
pub(super) struct Velocity(Vec3);

#[derive(Component, Clone, Copy, PartialEq, Default, Debug)]
pub(super) struct Acceleration(Vec3);

#[derive(Event, Debug)]
pub(super) struct SetAccelerationEvent {
    entity: Entity,
    acceleration: Acceleration,
}

#[derive(Event, Debug)]
pub(super) struct CollisionEvent {
    entity: Entity,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
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

impl RigidBody {
    pub(super) fn new(width: f32, height: f32) -> Self {
        Self(Cuboid::new(width, height, width))
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

impl CollisionEvent {
    pub(super) fn new(entity: Entity, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Self {
        Self { entity, x, y, z }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetAccelerationEvent>()
            .add_event::<CollisionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    (Self::set_accelerations, Self::remove_negligible_velocities),
                    Self::apply_accelerations,
                    Self::check_for_collisions,
                    Self::apply_velocities,
                    (Self::handle_collisions, Self::apply_drag),
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

    pub(super) fn check_for_collisions(
        query: Query<(Entity, &PhysicalPosition, &RigidBody, &Velocity)>,
        time: Res<Time>,
        chunks: Res<Chunks>,
        mut events: EventWriter<CollisionEvent>,
    ) {
        let delta_seconds = time.delta_seconds();

        for (entity, pos, body, vel) in &query {
            if vel.0 == Vec3::ZERO {
                continue;
            }

            let mut pos = pos.current;
            let displacement = vel.0 * delta_seconds;

            let collision_y = collision_at::<'Y'>(&mut pos, body, displacement, &chunks);
            let (collision_x, collision_z) = if displacement.z.abs() > displacement.x.abs() {
                let x = collision_at::<'X'>(&mut pos, body, displacement, &chunks);
                let z = collision_at::<'Z'>(&mut pos, body, displacement, &chunks);
                (x, z)
            } else {
                let z = collision_at::<'Z'>(&mut pos, body, displacement, &chunks);
                let x = collision_at::<'X'>(&mut pos, body, displacement, &chunks);
                (x, z)
            };

            if collision_x.is_some() || collision_y.is_some() || collision_z.is_some() {
                events.send(CollisionEvent::new(
                    entity,
                    collision_x,
                    collision_y,
                    collision_z,
                ));
            }
        }
    }

    fn apply_velocities(mut query: Query<(&Velocity, &mut PhysicalPosition)>, time: Res<Time>) {
        let delta_seconds = time.delta_seconds();

        for (velocity, mut position) in query.iter_mut() {
            position.previous = Some(position.current);
            position.current += *velocity * delta_seconds;
        }
    }

    fn handle_collisions(
        mut query: Query<(&mut PhysicalPosition, &mut Velocity), With<RigidBody>>,
        mut events: EventReader<CollisionEvent>,
    ) {
        for ev in events.read() {
            if let Ok((mut pos, mut vel)) = query.get_mut(ev.entity) {
                if let Some(x) = ev.x {
                    pos.current.x = x;
                    vel.0.x = 0.0;
                }
                if let Some(y) = ev.y {
                    pos.current.y = y;
                    vel.0.y = 0.0;
                }
                if let Some(z) = ev.z {
                    pos.current.z = z;
                    vel.0.z = 0.0;
                }
            }
        }
    }

    fn apply_drag(mut query: Query<&mut Velocity>) {
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

fn collision_at<const AXIS: char>(
    pos: &mut Vec3,
    body: &RigidBody,
    displacement: Vec3,
    chunks: &Chunks,
) -> Option<f32> {
    let body_center = *pos + Vec3::new(0.0, body.0.half_size.y, 0.0);
    let body_aabb = body.0.aabb_3d(body_center, Quat::IDENTITY);

    let (vel, displacement) = match AXIS {
        'X' => (displacement.x, Vec3::new(displacement.x, 0.0, 0.0)),
        'Y' => (displacement.y, Vec3::new(0.0, displacement.y, 0.0)),
        'Z' => (displacement.z, Vec3::new(0.0, 0.0, displacement.z)),
        _ => unreachable!(),
    };

    if vel == 0.0 {
        return None;
    }

    *pos += displacement;

    let min = Vec3::from(body_aabb.min) + displacement.min(Vec3::ZERO);
    let max = Vec3::from(body_aabb.max) + displacement.max(Vec3::ZERO);

    let mut collision_dist = vel.abs();
    let mut at = None;

    for x in min.x.floor() as i32..max.x.ceil() as i32 {
        for z in min.z.floor() as i32..max.z.ceil() as i32 {
            for y in min.y.floor() as i32..max.y.ceil() as i32 {
                if chunks
                    .block_at(IVec3::new(x, y, z))
                    .is_some_and(|block| block.is_solid())
                {
                    let (block_pos, body_pos) = match AXIS {
                        'X' => (
                            x as f32 + 0.5 - 0.5_f32.copysign(vel),
                            body_center.x + body.0.half_size.x.copysign(vel),
                        ),
                        'Y' => (
                            y as f32 + 0.5 - 0.5_f32.copysign(vel),
                            body_center.y + body.0.half_size.y.copysign(vel),
                        ),
                        'Z' => (
                            z as f32 + 0.5 - 0.5_f32.copysign(vel),
                            body_center.z + body.0.half_size.z.copysign(vel),
                        ),
                        _ => unreachable!(),
                    };

                    let dist = (body_pos - block_pos).abs();
                    if dist < collision_dist {
                        at = Some(match AXIS {
                            'X' => {
                                pos.x = block_pos - body.0.half_size.x.copysign(vel);
                                pos.x
                            }
                            'Y' => {
                                pos.y = block_pos
                                    - body.0.half_size.y.copysign(vel)
                                    - body.0.half_size.y;
                                pos.y
                            }
                            'Z' => {
                                pos.z = block_pos - body.0.half_size.z.copysign(vel);
                                pos.z
                            }
                            _ => unreachable!(),
                        });
                        collision_dist = dist;
                    }
                } else {
                    continue;
                }
            }
        }
    }
    at
}
