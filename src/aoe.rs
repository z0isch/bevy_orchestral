use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    metronome::{Metronome, nanos_per_beat},
    player::Player,
};

#[derive(Component, Debug)]
pub struct Aoe {
    initial_radius: f32,
    final_radius: f32,
    for_num_beats: u8,
    timer: Timer,
}

#[derive(Bundle)]
pub struct AoeBundle {
    aoe: Aoe,
    transform: Transform,
}

pub fn aoe_bundle(
    metronome: &Metronome,
    initial_radius: f32,
    final_radius: f32,
    for_num_beats: u8,
) -> AoeBundle {
    AoeBundle {
        aoe: Aoe {
            initial_radius,
            final_radius,
            for_num_beats,
            #[allow(clippy::cast_precision_loss)]
            timer: Timer::new(
                Duration::from_nanos(nanos_per_beat(metronome.bpm) * u64::from(for_num_beats)),
                TimerMode::Repeating,
            ),
        },
        transform: Transform::from_xyz(0., 0., 2.),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn aoe_system(
    metronome: Res<Metronome>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut aoe_query: Query<(Entity, &mut Aoe, &Transform)>,
) {
    for (entity, mut aoe, _) in &mut aoe_query {
        commands.entity(entity).try_insert_if_new((
            Mesh2d(meshes.add(Circle::new(aoe.initial_radius))),
            MeshMaterial2d(materials.add(Color::hsva(0., 0., 1., 0.1))),
            Collider::ball(aoe.initial_radius),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
        aoe.timer.tick(time.delta());
        if aoe.timer.just_finished() {
            commands.entity(entity).try_despawn();
        } else {
            let radius_diff = aoe.final_radius - aoe.initial_radius;
            #[allow(clippy::cast_precision_loss)]
            let total_nanos = nanos_per_beat(metronome.bpm) * u64::from(aoe.for_num_beats);
            let nanos_so_far = aoe.timer.elapsed().as_nanos();
            #[allow(clippy::cast_precision_loss)]
            let progress = nanos_so_far as f32 / total_nanos as f32;
            let radius = radius_diff.mul_add(progress, aoe.initial_radius);

            commands.entity(entity).try_insert((
                Mesh2d(meshes.add(Circle::new(radius as f32))),
                Collider::ball(radius),
            ));
        }
    }
}

#[derive(Component)]
pub struct AoeDuration {
    pub velocity: f32,
    pub timer: Timer,
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_aoe_duration(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AoeDuration, &Transform, &mut Velocity)>,
    mut player_query: Query<&Transform, With<Player>>,
) {
    for (entity, mut aoe_duration, transform, mut velocity) in &mut query {
        aoe_duration.timer.tick(time.delta());
        if aoe_duration.timer.just_finished() {
            commands.entity(entity).try_remove::<AoeDuration>();
            velocity.linvel = Vec2::ZERO;
        } else if let Ok(player_transform) = player_query.single_mut() {
            let direction = (transform.translation.xy() - player_transform.translation.xy())
                .normalize_or_zero();
            velocity.linvel = direction * aoe_duration.velocity;
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn aoe_collision_system(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    query_aoe: Query<(Entity, &Aoe, &Transform)>,
    query_enemy: Query<(Entity, &Enemy, &Transform)>,
) {
    let velocity = 60.;
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let enemy_entity =
                if query_aoe.get(*entity1).is_ok() && query_enemy.get(*entity2).is_ok() {
                    *entity2
                } else if query_aoe.get(*entity2).is_ok() && query_enemy.get(*entity1).is_ok() {
                    *entity1
                } else {
                    continue;
                };

            commands.entity(enemy_entity).try_insert(AoeDuration {
                velocity,
                timer: Timer::new(
                    Duration::from_nanos(nanos_per_beat(metronome.bpm)) * 12,
                    TimerMode::Once,
                ),
            });
        }
    }
}
