use bevy::prelude::*;
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

#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_enemy_system(
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

    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished()
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
pub fn enemy_movement_system(
    mut commands: Commands,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(Entity, &MovementSpeed, &Transform), With<Enemy>>,
) {
    if metronome.started
        && metronome.is_beat_start_frame
        && is_down_beat(&metronome)
        && let Ok(player_transform) = player_query.single()
    {
        for (entity, movement_speed, enemy_transform) in &mut enemy_query {
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
