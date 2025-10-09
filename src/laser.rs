use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    health::Health,
    metronome::{Metronome, MetronomeTimer},
};

#[derive(Component)]
pub struct Laser {
    damage_per_beat: u128,
    timer: MetronomeTimer,
    entities_damaged_on_beat: HashMap<u8, HashSet<Entity>>,
}

#[derive(Bundle)]
pub struct LaserBundle {
    laser: Laser,
    mesh: Mesh2d,
    active_events: ActiveEvents,
    sensor: Sensor,
    material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    rigid_body: RigidBody,
    transform: Transform,
}

pub fn laser_bundle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    damage_per_beat: u128,
    number_beats_duration: u8,
    width: f32,
    length: f32,
) -> LaserBundle {
    LaserBundle {
        laser: Laser {
            damage_per_beat,
            timer: MetronomeTimer::new(number_beats_duration),
            entities_damaged_on_beat: HashMap::new(),
        },
        mesh: Mesh2d(meshes.add(Rectangle::new(width, length))),
        material: MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
        collider: Collider::cuboid(width / 2., length / 2.),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        rigid_body: RigidBody::KinematicVelocityBased,
        active_events: ActiveEvents::COLLISION_EVENTS,
        transform: Transform::from_xyz(0., 0., 1.),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn laser_system(
    rapier_context: ReadRapierContext,
    metronome: Res<Metronome>,
    time: Res<Time>,
    mut commands: Commands,
    mut query_laser: Query<(Entity, &mut Laser)>,
    mut query_enemy: Query<(Entity, &mut Health), With<Enemy>>,
) {
    let rapier_context = rapier_context.single().unwrap();
    for (laser_entity, mut laser) in &mut query_laser {
        laser.timer.tick(&metronome, *time);
        if laser.timer.just_finished(&metronome) {
            commands.entity(laser_entity).despawn();
        } else if !laser.timer.finished() {
            for (enemy_entity, mut health) in &mut query_enemy {
                if rapier_context.intersection_pair(laser_entity, enemy_entity) == Some(true) {
                    let beats_elapsed = laser.timer.beats_elapsed();
                    let entities_damaged = laser
                        .entities_damaged_on_beat
                        .entry(beats_elapsed)
                        .or_insert_with(HashSet::new);
                    if entities_damaged.insert(enemy_entity) && health.current_health > 0 {
                        health.current_health -= laser.damage_per_beat;
                    }
                }
            }
        }
    }
}
