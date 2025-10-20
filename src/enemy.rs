use bevy::{log, prelude::*};
use bevy_aseprite_ultra::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{Rng, rng};

use crate::{
    MovementSpeed,
    bounce::initial_bounce,
    health::{Health, health_bar_bundle},
    metronome::{Metronome, is_down_beat},
    player::Player,
    slide::initial_slide,
};

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Raccoon {
    min_distance_squared_to_player: f32,
    bullet_radius: f32,
    bullet_velocity: f32,
}

#[derive(Component, Debug)]
pub struct RaccoonBullet {
    velocity: f32,
    direction: Vec2,
}

#[derive(Component, Debug)]
pub struct Skunk;

#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub skunk_timer: Timer,
    pub raccoon_timer: Timer,
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_skunk_system(
    mut commands: Commands,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    if !metronome.started {
        return;
    }

    spawn_timer.skunk_timer.tick(time.delta());

    if spawn_timer.skunk_timer.just_finished()
        && let Ok(player_transform) = player_query.single()
    {
        let mut rng = rng();
        let angle = rng.random::<f32>() * std::f32::consts::TAU;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 100.0;
        let spawn_pos = player_transform.translation.xy() + offset;

        let enemy_sprite_scale = 0.3;
        let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
        sprite_transform.scale = Vec3::new(enemy_sprite_scale, enemy_sprite_scale, 0.);

        commands.spawn((
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Collider::ball((45.0 / 2.0) * enemy_sprite_scale),
            MovementSpeed(6.),
            Enemy,
            Skunk,
            Velocity::zero(),
            Health {
                max_health: 5,
                current_health: 5,
            },
            Visibility::default(),
            children![
                health_bar_bundle(),
                (
                    AseAnimation {
                        animation: Animation::tag("idle-right")
                            .with_repeat(AnimationRepeat::Loop)
                            .with_direction(AnimationDirection::Forward)
                            .with_speed(0.5),
                        aseprite: asset_server.load("sprites/skunk.aseprite"),
                    },
                    Sprite::default(),
                    sprite_transform,
                    initial_bounce(1.2)
                )
            ],
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn skunk_movement_system(
    mut commands: Commands,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    skunk_query: Query<(Entity, &MovementSpeed, &Transform), With<Skunk>>,
) {
    if metronome.started
        && metronome.is_beat_start_frame
        && is_down_beat(&metronome)
        && let Ok(player_transform) = player_query.single()
    {
        for (entity, movement_speed, enemy_transform) in skunk_query {
            let mut rng = rng();
            let speed_variation = rng.random_range(-0.2..=0.2);
            let varied_velocity = movement_speed.0 * (1.0 + speed_variation);
            commands.entity(entity).try_insert(initial_slide(
                varied_velocity,
                player_transform.translation.xy() - enemy_transform.translation.xy(),
                1,
                &metronome,
            ));
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_raccoon_system(
    mut commands: Commands,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    if !metronome.started {
        return;
    }

    spawn_timer.raccoon_timer.tick(time.delta());

    if spawn_timer.raccoon_timer.just_finished()
        && let Ok(player_transform) = player_query.single()
    {
        let mut rng = rng();
        let angle = rng.random::<f32>() * std::f32::consts::TAU;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 100.0;
        let spawn_pos = player_transform.translation.xy() + offset;

        let enemy_sprite_scale = 0.3;
        let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
        sprite_transform.scale = Vec3::new(enemy_sprite_scale, enemy_sprite_scale, 0.);

        commands.spawn((
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Collider::ball((45.0 / 2.0) * enemy_sprite_scale),
            MovementSpeed(6.),
            Enemy,
            Raccoon {
                min_distance_squared_to_player: 100.0 * 100.0,
                bullet_radius: 5.0,
                bullet_velocity: 30.0,
            },
            Velocity::zero(),
            Health {
                max_health: 5,
                current_health: 5,
            },
            Visibility::default(),
            children![
                health_bar_bundle(),
                (
                    AseAnimation {
                        animation: Animation::tag("idle-right")
                            .with_repeat(AnimationRepeat::Loop)
                            .with_direction(AnimationDirection::Forward)
                            .with_speed(0.5),
                        aseprite: asset_server.load("sprites/raccoon.aseprite"),
                    },
                    Sprite::default(),
                    sprite_transform,
                    initial_bounce(1.2)
                )
            ],
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn raccoon_movement_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    raccoon_query: Query<(Entity, &MovementSpeed, &Transform, &Raccoon)>,
) {
    if metronome.started
        && metronome.is_beat_start_frame
        && is_down_beat(&metronome)
        && let Ok(player_transform) = player_query.single()
    {
        for (entity, movement_speed, raccoon_transform, raccoon) in raccoon_query {
            let distance_squared_to_player = player_transform
                .translation
                .distance_squared(raccoon_transform.translation);
            let mut rng = rng();
            if distance_squared_to_player < raccoon.min_distance_squared_to_player {
                let speed_variation = rng.random_range(-0.2..=0.2);
                let varied_velocity = movement_speed.0 * (1.0 + speed_variation);
                let away_from_player =
                    raccoon_transform.translation.xy() - player_transform.translation.xy();
                commands.entity(entity).try_insert(initial_slide(
                    varied_velocity,
                    away_from_player,
                    1,
                    &metronome,
                ));
            } else {
                let towards_player =
                    player_transform.translation.xy() - raccoon_transform.translation.xy();
                let should_shoot = rng.random_range(0.0..=1.0) < 0.25;
                if should_shoot {
                    commands.spawn((
                        RaccoonBullet {
                            velocity: raccoon.bullet_velocity,
                            direction: towards_player,
                        },
                        Transform::from_xyz(
                            raccoon_transform.translation.x,
                            raccoon_transform.translation.y,
                            2.,
                        ),
                        Velocity::zero(),
                        Mesh2d(meshes.add(Circle::new(raccoon.bullet_radius))),
                        MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
                        Collider::ball(raccoon.bullet_radius),
                        CollisionGroups::new(Group::GROUP_1, Group::ALL),
                        Sensor,
                        RigidBody::KinematicVelocityBased,
                    ));
                } else {
                    let should_move = rng.random_range(0.0..=1.0) < 0.25;
                    if should_move {
                        commands.entity(entity).try_insert(initial_slide(
                            movement_speed.0,
                            towards_player,
                            1,
                            &metronome,
                        ));
                    }
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn raccoon_bullet_system(
    metronome: Res<Metronome>,
    mut commands: Commands,
    bullet_query: Query<(Entity, &RaccoonBullet)>,
) {
    if metronome.started && metronome.is_beat_start_frame && is_down_beat(&metronome) {
        for (bullet_entity, bullet) in bullet_query {
            commands.entity(bullet_entity).try_insert(initial_slide(
                bullet.velocity,
                bullet.direction,
                1,
                &metronome,
            ));
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn raccoon_bullet_collision_system(
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    bullet_query: Query<Entity, With<RaccoonBullet>>,
    player_query: Query<Entity, With<Player>>,
) {
    for bullet_entity in bullet_query {
        let rapier_context = rapier_context.single().unwrap();
        for player_entity in player_query {
            if rapier_context.intersection_pair(bullet_entity, player_entity) == Some(true) {
                commands.entity(bullet_entity).try_despawn();
            }
        }
    }
}
