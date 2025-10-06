use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    metronome::{Metronome, nanos_per_beat},
    player::Player,
};

#[derive(Component, Debug)]
pub struct AOE {
    initial_radius: f32,
    final_radius: f32,
    for_num_beats: u8,
    timer: Timer,
}

#[derive(Bundle)]
pub struct AOEBundle {
    aoe: AOE,
    mesh: Mesh2d,
    active_events: ActiveEvents,
    sensor: Sensor,
    material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    transform: Transform,
}

pub fn aoe_bundle(
    metronome: &Metronome,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    initial_radius: f32,
    final_radius: f32,
    for_num_beats: u8,
) -> AOEBundle {
    AOEBundle {
        aoe: AOE {
            initial_radius,
            final_radius,
            for_num_beats,
            timer: Timer::new(
                Duration::from_nanos(nanos_per_beat(metronome.bpm) * for_num_beats as u64),
                TimerMode::Repeating,
            ),
        },
        mesh: Mesh2d(meshes.add(Circle::new(initial_radius))),
        material: MeshMaterial2d(materials.add(Color::hsva(0., 0., 1., 0.1))),
        collider: Collider::ball(initial_radius),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        active_events: ActiveEvents::COLLISION_EVENTS,
        transform: Transform::from_xyz(0., 0., 0.),
    }
}

pub fn aoe_system(
    metronome: Res<Metronome>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut AOE)>,
) {
    for (entity, mut aoe) in query.iter_mut() {
        aoe.timer.tick(time.delta());
        if aoe.timer.just_finished() {
            commands.entity(entity).despawn();
        } else {
            let radius_diff = aoe.final_radius - aoe.initial_radius;
            let total_nanos = nanos_per_beat(metronome.bpm) * aoe.for_num_beats as u64;
            let nanos_so_far = aoe.timer.elapsed().as_nanos();
            let progress = nanos_so_far as f32 / total_nanos as f32;
            let radius = aoe.initial_radius + (radius_diff * progress);

            commands.entity(entity).insert((
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

pub fn process_aoe_duration(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AoeDuration, &Transform, &mut Velocity)>,
    mut player_query: Query<&Transform, With<Player>>,
) {
    for (entity, mut aoe_duration, transform, mut velocity) in query.iter_mut() {
        aoe_duration.timer.tick(time.delta());
        if aoe_duration.timer.just_finished() {
            commands.entity(entity).remove::<AoeDuration>();
            velocity.linvel = Vec2::ZERO;
        } else {
            if let Ok(player_transform) = player_query.single_mut() {
                let direction = (transform.translation.xy() - player_transform.translation.xy())
                    .normalize_or_zero();
                velocity.linvel = direction * aoe_duration.velocity;
            }
        }
    }
}

pub fn aoe_collision_system(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    query_aoe: Query<(Entity, &AOE, &Transform)>,
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

            commands.entity(enemy_entity).insert(AoeDuration {
                velocity,
                timer: Timer::new(
                    Duration::from_nanos(nanos_per_beat(metronome.bpm)) * 12,
                    TimerMode::Once,
                ),
            });
        }
    }
}
