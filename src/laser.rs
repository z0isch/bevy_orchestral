use std::{
    collections::{HashMap, HashSet},
    f32::consts::FRAC_PI_2,
};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    health::Health,
    metronome::{Metronome, MetronomeTimer},
    nearest_enemy::NearestEnemy,
};

#[derive(Component)]
pub struct Laser {
    damage_per_beat: u128,
    timer: MetronomeTimer,
    entities_damaged_on_beat: HashMap<u8, HashSet<Entity>>,
    target: Option<Entity>,
    length: f32,
    width: f32,
}

#[derive(Bundle)]
pub struct LaserBundle {
    laser: Laser,
    nearest_enemy: NearestEnemy,
    transform: Transform,
}

pub fn laser_bundle(
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
            target: None,
            length,
            width,
        },
        nearest_enemy: NearestEnemy(None),
        transform: Transform::from_xyz(0., 0., 1.),
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub fn laser_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_context: ReadRapierContext,
    metronome: Res<Metronome>,
    time: Res<Time>,
    mut commands: Commands,
    mut laser_query: Query<(Entity, &mut Laser, &NearestEnemy, &mut Transform, &ChildOf)>,
    parent_query: Query<&Transform, (With<Children>, (Without<Enemy>, Without<Laser>))>,
    mut enemy_query: Query<(Entity, &mut Health, &Transform), (With<Enemy>, Without<Laser>)>,
) {
    let rapier_context = rapier_context.single().unwrap();
    for (laser_entity, mut laser, nearest_enemy, mut laser_transform, parent) in &mut laser_query {
        if laser.target.is_none()
            && let Some(nearest_enemy) = nearest_enemy.0
            && let Ok((_, _, enemy_transform)) = enemy_query.get(nearest_enemy)
            && let Ok(parent_transform) = parent_query.get(parent.parent())
        {
            laser.target = Some(nearest_enemy);
            let direction = (enemy_transform.translation - parent_transform.translation)
                .xy()
                .normalize_or_zero();

            laser_transform.rotate(Quat::from_rotation_z(
                direction.y.atan2(direction.x) + FRAC_PI_2,
            ));
            laser_transform.translation = direction.extend(1.) * laser.length / 2.;

            commands.entity(laser_entity).insert_if_new((
                Mesh2d(meshes.add(Rectangle::new(laser.width, laser.length))),
                MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 0.5))),
                Collider::cuboid(laser.width / 2., laser.length / 2.),
                CollisionGroups::new(Group::GROUP_2, Group::ALL),
                Sensor,
                RigidBody::KinematicVelocityBased,
            ));
        }

        if laser.target.is_some() {
            laser.timer.tick(&metronome, *time);
            if laser.timer.just_finished(&metronome) {
                commands.entity(laser_entity).despawn();
            } else if !laser.timer.finished() {
                for (enemy_entity, mut health, _) in &mut enemy_query {
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
        } else {
            commands.entity(laser_entity).despawn();
        }
    }
}
