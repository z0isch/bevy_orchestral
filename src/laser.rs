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
    shooter: Entity,
    direction: Option<Vec2>,
    length: f32,
}

#[derive(Bundle)]
pub struct LaserBundle {
    laser: Laser,
    audio_player: AudioPlayer,
    mesh: Mesh2d,
    mesh_material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    sensor: Sensor,
    rigid_body: RigidBody,
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

#[allow(clippy::too_many_arguments)]
pub fn laser_bundle(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    laser_sfx: &Res<LaserSFX>,
    damage_per_beat: u128,
    number_beats_duration: u8,
    width: f32,
    length: f32,
    shooter: Entity,
) -> LaserBundle {
    LaserBundle {
        laser: Laser {
            damage_per_beat,
            timer: MetronomeTimer::new(number_beats_duration),
            entities_damaged_on_beat: HashMap::new(),
            direction: None,
            length,
            shooter,
        },
        audio_player: AudioPlayer::new(laser_sfx.fire.clone()),
        mesh: Mesh2d(meshes.add(Rectangle::new(width, length))),
        mesh_material: MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 0.5))),
        collider: Collider::cuboid(width / 2., length / 2.),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        rigid_body: RigidBody::KinematicVelocityBased,
    }
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub fn laser_system(
    rapier_context: ReadRapierContext,
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut laser_query: Query<(Entity, &mut Laser)>,
    shooter_query: Query<&Transform, (Without<Enemy>, Without<Laser>)>,
    mut enemy_query: Query<(Entity, &mut Health, &Transform), (With<Enemy>, Without<Laser>)>,
) {
    let rapier_context = rapier_context.single().unwrap();
    for (laser_entity, mut laser) in &mut laser_query {
        if let Ok(shooter_transform) = shooter_query.get(laser.shooter) {
            let direction = laser.direction.get_or_insert_with(|| {
                if let Some((_, enemy_transform)) =
                    find_nearest_entity(*shooter_transform, enemy_query.transmute_lens().query())
                {
                    (enemy_transform.translation - shooter_transform.translation)
                        .xy()
                        .normalize_or_zero()
                } else {
                    Vec2::from_angle(rng().random_range(0.0..std::f32::consts::TAU))
                }
            });

            let mut laser_transform = *shooter_transform;
            laser_transform.rotate(Quat::from_rotation_z(
                direction.y.atan2(direction.x) + FRAC_PI_2,
            ));
            laser_transform.translation += direction.extend(1.) * laser.length / 2.;
            commands.entity(laser_entity).try_insert(laser_transform);
        }

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
