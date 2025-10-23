use std::{
    collections::{HashMap, HashSet},
    f32::consts::FRAC_PI_2,
};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{Rng, rng};

use crate::{
    enemy::Enemy,
    health::Health,
    metronome::{Metronome, MetronomeTimer},
    nearest_entity::find_nearest_entity,
};

#[derive(Component, Debug)]
pub struct Laser {
    damage_per_beat: u128,
    timer: MetronomeTimer,
    entities_damaged_on_beat: HashMap<u8, HashSet<Entity>>,
    transform: Option<Transform>,
    length: f32,
    width: f32,
}

#[derive(Bundle)]
pub struct LaserBundle {
    laser: Laser,
    transform: Transform,
    audio_player: AudioPlayer,
}

#[derive(Resource)]
pub struct LaserSFX {
    pub fire: Handle<AudioSource>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn setup_laser_sfx(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(LaserSFX {
        fire: asset_server.load("sounds/laser.ogg"),
    });
}

pub fn laser_bundle(
    laser_sfx: &Res<LaserSFX>,
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
            transform: None,
            length,
            width,
        },
        transform: Transform::from_xyz(0., 0., 2.),
        audio_player: AudioPlayer::new(laser_sfx.fire.clone()),
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub fn laser_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_context: ReadRapierContext,
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut laser_query: Query<(Entity, &mut Laser, &mut Transform, &ChildOf)>,
    parent_query: Query<&Transform, (With<Children>, (Without<Enemy>, Without<Laser>))>,
    mut enemy_query: Query<(Entity, &mut Health, &Transform), (With<Enemy>, Without<Laser>)>,
) {
    let rapier_context = rapier_context.single().unwrap();
    for (laser_entity, mut laser, mut laser_transform, parent) in &mut laser_query {
        if let Some(transform) = laser.transform {
            if transform != *laser_transform {
                *laser_transform = transform;
            }
        } else {
            let direction = if let Ok(parent_transform) = parent_query.get(parent.parent())
                && let Some((_, enemy_transform)) =
                    find_nearest_entity(*parent_transform, enemy_query.transmute_lens().query())
            {
                (enemy_transform.translation - parent_transform.translation)
                    .xy()
                    .normalize_or_zero()
            } else {
                Vec2::from_angle(rng().random_range(0.0..std::f32::consts::TAU))
            };

            laser_transform.rotate(Quat::from_rotation_z(
                direction.y.atan2(direction.x) + FRAC_PI_2,
            ));
            laser_transform.translation = direction.extend(1.) * laser.length / 2.;
            laser.transform = Some(*laser_transform);
        }

        commands.entity(laser_entity).try_insert_if_new((
            Mesh2d(meshes.add(Rectangle::new(laser.width, laser.length))),
            MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 0.5))),
            Collider::cuboid(laser.width / 2., laser.length / 2.),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
            Sensor,
            RigidBody::KinematicVelocityBased,
        ));

        laser.timer.tick(&metronome);
        if laser.timer.just_finished(&metronome) {
            commands.entity(laser_entity).try_despawn();
        } else if !laser.timer.finished() {
            for (enemy_entity, mut health, _) in &mut enemy_query {
                if rapier_context.intersection_pair(laser_entity, enemy_entity) == Some(true) {
                    let beats_elapsed = laser.timer.beats_elapsed();
                    let entities_damaged = laser
                        .entities_damaged_on_beat
                        .entry(beats_elapsed)
                        .or_insert_with(HashSet::new);
                    if entities_damaged.insert(enemy_entity) {
                        health.current_health =
                            health.current_health.saturating_sub(laser.damage_per_beat);
                    }
                }
            }
        }
    }
}
