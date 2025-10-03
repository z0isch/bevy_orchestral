mod aoe;
mod bounce;
mod map;
mod metronome;
mod player;
mod slide;
mod window_size;

use std::time::Duration;

use bevy::{
    asset::AssetMetaCheck, input::common_conditions::input_toggle_active, log, prelude::*,
    time::Stopwatch, window::WindowResolution,
};
use bevy_aseprite_ultra::{
    AsepriteUltraPlugin,
    prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation},
};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, ReadRapierContext},
    prelude::{
        ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionEvent, CollisionGroups,
        Damping, Dominance, ExternalForce, ExternalImpulse, Group, KinematicCharacterController,
        LockedAxes, QueryFilterFlags, RigidBody, Sensor, Sleeping, Velocity,
    },
    render::RapierDebugRenderPlugin,
};
use fraction::Fraction;
use rand::{Rng, rng};

use crate::{
    aoe::{AOE, AoeDuration, aoe_bundle, aoe_system, process_aoe_duration},
    bounce::{bounce_system, initial_bounce, initial_tile_bounce, tile_bounce_system},
    map::setup_map,
    metronome::{
        Metronome, MetronomeTimer, down_beats, initial_metronome, is_down_beat, metronome_system,
        nanos_per_beat, within_nanos_window,
    },
    player::Player,
    slide::{Slide, initial_slide, slide_system},
    window_size::{WINDOW_HEIGHT, WINDOW_WIDTH, setup_window_size},
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(AsepriteUltraPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        //.add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_systems(
            Startup,
            (setup_window_size, setup, setup_map, set_gravity).chain(),
        )
        .add_systems(First, metronome_system)
        .add_systems(Update, tile_bounce_system)
        .add_systems(Update, toggle_audio)
        .add_systems(
            Update,
            (
                control_player,
                slide_system,
                bounce_system,
                spawn_enemy_system,
                player_animation,
            )
                .chain(),
        )
        .add_systems(Update, enemy_movement_system)
        .add_systems(Update, (aoe_system, process_aoe_duration))
        .add_systems(Update, display_events)
        .run();
}

fn set_gravity(rapier_config: Query<&mut RapierConfiguration>) {
    let rapier_config = rapier_config.single_inner();
    match rapier_config {
        Ok(mut rapier_config) => rapier_config.gravity = Vec2::ZERO,
        Err(_) => {
            log::info!("No RapierConfiguration found");
        }
    }
}

#[derive(Component, Debug)]
struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(101));
    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(0.3, TimerMode::Repeating),
    });
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>("sounds/song-101bpm.ogg")),
        PlaybackSettings::default().paused(),
    ));

    let player_sprite_scale = 0.15;
    let mut transform = Transform::from_xyz(0., 0., 2.);
    transform.scale = Vec3::new(player_sprite_scale, player_sprite_scale, 1.);
    commands.spawn((
        transform,
        AseAnimation {
            animation: Animation::tag("idle-right")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.5),
            aseprite: asset_server.load("sprites/maestro.aseprite"),
        },
        Sprite::default(),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::ONLY_FIXED,
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule_y(100., 25.),
        MovementSpeed(1.),
        initial_bounce(1.1),
        Player,
        Velocity::zero(),
    ));
}

fn toggle_audio(
    mut audio_sink: Query<&AudioSink>,
    mut metronome: ResMut<Metronome>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        if let Ok(audio_sink) = audio_sink.single_mut() {
            if metronome.started {
                audio_sink.pause();
                metronome.started = false;
            } else {
                audio_sink.play();
                metronome.started = true;
            }
        }
    }
}

#[derive(Component, Debug)]
struct MovementSpeed(f32);

fn player_animation(
    mut animation_query: Query<(
        &mut AseAnimation,
        &KinematicCharacterController,
        &mut Transform,
    )>,
) {
    for (mut ase_sprite_animation, kinematic_character_controller, mut transform) in
        animation_query.iter_mut()
    {
        let velocity = kinematic_character_controller
            .translation
            .unwrap_or(Vec2::ZERO);
        let near_idle = velocity.length_squared() < 0.1;
        if near_idle {
            transform.scale.x = transform.scale.x.abs();
            ase_sprite_animation.animation.play_loop("idle-right");
        } else {
            if velocity.x > 0. {
                if velocity.y > 0. {
                    transform.scale.x = transform.scale.x.abs() * -1.;
                    ase_sprite_animation.animation.play_loop("walk-up-left");
                } else {
                    transform.scale.x = transform.scale.x.abs();
                    ase_sprite_animation.animation.play_loop("walk-right");
                }
            } else {
                if velocity.y > 0. {
                    transform.scale.x = transform.scale.x.abs();
                    ase_sprite_animation.animation.play_loop("walk-up-left");
                } else {
                    transform.scale.x = transform.scale.x.abs() * -1.;
                    ase_sprite_animation.animation.play_loop("walk-right");
                }
            }
        }
    }
}

fn control_player(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<
        (
            Entity,
            &MovementSpeed,
            &Transform,
            &mut KinematicCharacterController,
        ),
        (With<Player>, Without<Slide>),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (entity, movement_speed, transform, mut kinematic_character_controller) in query.iter_mut()
    {
        if keyboard_input.just_pressed(KeyCode::KeyJ) {
            // let grace_period = Fraction::from(90u64 * 1_000_000);

            // if down_beats(&metronome)
            //     .iter()
            //     .any(|&beat| within_nanos_window(&metronome, beat, grace_period))
            // {
            commands.spawn(aoe_bundle(
                &metronome,
                &mut meshes,
                &mut materials,
                &transform,
                30.0,
                75.0,
                2,
            ));
            //}
        }
        if keyboard_input.just_pressed(KeyCode::KeyK) {
            // let grace_period = Fraction::from(90u64 * 1_000_000);

            // if down_beats(&metronome)
            //     .iter()
            //     .any(|&beat| within_nanos_window(&metronome, beat, grace_period))
            // {
            let mut current_direction = Vec2::new(0., 0.);
            if keyboard_input.pressed(KeyCode::KeyW) {
                current_direction.y = 1.;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                current_direction.y = -1.;
            }
            if keyboard_input.pressed(KeyCode::KeyA) {
                current_direction.x = -1.;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                current_direction.x = 1.;
            }
            if current_direction.length_squared() > 0. {
                commands.entity(entity).insert(initial_slide(
                    10.,
                    current_direction,
                    1,
                    &metronome,
                ));
            }
        }
        //}

        let mut velocity_desired = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity_desired.y = 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            velocity_desired.y = -1.;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity_desired.x = -1.;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity_desired.x = 1.;
        }
        kinematic_character_controller.translation =
            Some(velocity_desired.normalize_or_zero() * movement_speed.0);
    }
}

fn spawn_enemy_system(
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

    if spawn_timer.timer.just_finished() {
        if let Ok(player_transform) = player_query.single() {
            let mut rng = rng();
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
            let offset = Vec2::new(angle.cos(), angle.sin()) * 100.0;
            let spawn_pos = player_transform.translation.xy() + offset;

            let enemy_sprite_scale = 0.3;
            let mut transform = Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.);
            transform.scale = Vec3::new(enemy_sprite_scale, enemy_sprite_scale, 1.);

            commands.spawn((
                transform,
                AseAnimation {
                    animation: Animation::tag("idle-right")
                        .with_repeat(AnimationRepeat::Loop)
                        .with_direction(AnimationDirection::Forward)
                        .with_speed(0.5),
                    aseprite: asset_server.load("sprites/skunk.aseprite"),
                },
                Sprite::default(),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Collider::ball(45.0 / 2.0),
                MovementSpeed(6.),
                Enemy,
                initial_bounce(1.2),
                Velocity::zero(),
            ));
        }
    }
}

fn enemy_movement_system(
    mut commands: Commands,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(Entity, &MovementSpeed, &Transform), With<Enemy>>,
) {
    if metronome.started && metronome.is_beat_start_frame && is_down_beat(&metronome) {
        if let Ok(player_transform) = player_query.single() {
            for (entity, movement_speed, enemy_transform) in enemy_query.iter_mut() {
                let mut rng = rng();
                let speed_variation = rng.random_range(-0.2..=0.2);
                let varied_velocity = movement_speed.0 * (1.0 + speed_variation);
                commands.entity(entity).insert(initial_slide(
                    varied_velocity,
                    player_transform.translation.xy() - enemy_transform.translation.xy(),
                    1,
                    &metronome,
                ));
            }
        }
    }
}

fn display_events(
    metronome: Res<Metronome>,
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    query_aoe: Query<(Entity, &AOE, &Transform)>,
    query_enemy: Query<(Entity, &Enemy, &Transform)>,
) {
    let velocity = 60.;
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity, entity1, collision_event_flags) => {
                if let Ok((aoe_entity, aoe, aoe_transform)) = query_aoe.get(*entity) {
                    if let Ok((enemy_entity, enemy, enemy_transform)) = query_enemy.get(*entity1) {
                        commands.entity(enemy_entity).insert((AoeDuration {
                            velocity,
                            timer: Timer::new(
                                Duration::from_nanos(nanos_per_beat(metronome.bpm)) * 12,
                                TimerMode::Once,
                            ),
                        },));
                    }
                }
                if let Ok((aoe_entity, aoe, aoe_transform)) = query_aoe.get(*entity1) {
                    if let Ok((enemy_entity, enemy, enemy_transform)) = query_enemy.get(*entity) {
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
            CollisionEvent::Stopped(entity, entity1, collision_event_flags) => {}
        }
    }
}
