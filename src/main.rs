#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]
mod aoe;
mod bounce;
mod bullet;
mod enemy;
mod health;
mod laser;
mod map;
mod metronome;
mod note_highway;
mod player;
mod slide;
mod window_size;

use bevy::{
    asset::AssetMetaCheck, audio::Volume, input::common_conditions::input_toggle_active, log,
    prelude::*, window::WindowResolution,
};
use bevy_aseprite_ultra::{
    AsepriteUltraPlugin,
    prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation},
};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_enhanced_input::prelude::{Press, *};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    prelude::{
        Collider, CollisionGroups, Group, KinematicCharacterController, LockedAxes,
        QueryFilterFlags, RigidBody, Velocity,
    },
};
use fraction::Fraction;
use rand::{Rng, rng};

use crate::{
    aoe::{aoe_bundle, aoe_collision_system, aoe_system, process_aoe_duration},
    bounce::{bounce_system, initial_bounce, tile_bounce_system},
    bullet::{BulletSFX, bullet_bundle, bullet_collision_system, bullet_system, setup_bullet_sfx},
    enemy::Enemy,
    health::{
        Health, despawn_enemy_on_zero_health, health_bar_bundle, health_bar_system,
        on_health_bar_add,
    },
    laser::{LaserSFX, laser_bundle, laser_system, setup_laser_sfx},
    map::setup_map,
    metronome::{
        Metronome, down_beats, initial_metronome, is_down_beat, metronome_system,
        within_nanos_window,
    },
    note_highway::{
        beat_line_system, note_highway_system, on_beat_line_system, setup_note_highway,
    },
    player::Player,
    slide::{Slide, initial_slide, slide_system},
    window_size::{WINDOW_HEIGHT, WINDOW_WIDTH, setup_window_size},
};

const SONG_BPM: u64 = 101;
const SONG_FILE: &str = "sounds/song-101bpm.ogg";

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
        .add_plugins(EnhancedInputPlugin)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_input_context::<Player>()
        .add_systems(
            Startup,
            (
                setup_window_size,
                setup,
                setup_map,
                set_gravity,
                setup_note_highway,
                setup_laser_sfx,
                setup_bullet_sfx,
            )
                .chain(),
        )
        .add_systems(PreUpdate, metronome_system)
        .add_systems(
            Update,
            (
                note_highway_system,
                on_beat_line_system,
                beat_line_system,
                tile_bounce_system,
                bounce_system,
                aoe_system,
                aoe_collision_system,
                process_aoe_duration,
                bullet_system,
                laser_system,
                despawn_enemy_on_zero_health,
                health_bar_system,
                spawn_enemy_system,
                bullet_collision_system,
                enemy_movement_system,
                (slide_system, player_animation).chain(),
            ),
        )
        .add_observer(on_health_bar_add)
        .add_observer(apply_movement)
        .add_observer(toggle_audio)
        .add_observer(toggle_muted)
        .add_observer(apply_slide)
        .add_observer(apply_laser)
        .add_observer(apply_bullet)
        .add_observer(apply_aoe)
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

#[derive(Resource)]
struct GracePeriod(Fraction);

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

#[allow(clippy::needless_pass_by_value)]
fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(SONG_BPM));
    commands.insert_resource(GracePeriod(Fraction::from(90u64 * 1_000_000)));
    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
    });
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>(SONG_FILE)),
        PlaybackSettings::default().paused(),
    ));

    let player_sprite_scale = 0.15;
    let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
    sprite_transform.scale = Vec3::new(player_sprite_scale, player_sprite_scale, 0.);
    commands.spawn((
        Transform::from_xyz(0., 0., 2.),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::ONLY_FIXED,
            filter_groups: Some(CollisionGroups::new(
                Group::GROUP_1,
                Group::ALL - Group::GROUP_2,
            )),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule_y(100. * player_sprite_scale, 25. * player_sprite_scale),
        MovementSpeed(0.5),
        Player,
        Velocity::zero(),
        Visibility::default(),
        children![(
            sprite_transform,
            AseAnimation {
                animation: Animation::tag("idle-right")
                    .with_repeat(AnimationRepeat::Loop)
                    .with_direction(AnimationDirection::Forward)
                    .with_speed(1.5),
                aseprite: asset_server.load("sprites/maestro.aseprite"),
            },
            Sprite::default(),
            initial_bounce(1.1)
        )],
        actions!(Player[(
            Action::<Movement>::new(),
            DeadZone::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
            )),
        ),(
            Action::<ToggleAudio>::new(),
            Press::default(),
            bindings![KeyCode::KeyX, GamepadButton::Start],
        ),(
            Action::<ToggleMuted>::new(),
            Press::default(),
            bindings![KeyCode::KeyX, GamepadButton::Select],
        ),(
            Action::<SlideInputAction>::new(),
            Press::default(),
            bindings![KeyCode::Space, GamepadButton::LeftTrigger],
        ),(
            Action::<LaserInputAction>::new(),
            Press::default(),
            bindings![KeyCode::ArrowUp, GamepadButton::North],
        ),(
            Action::<BulletInputAction>::new(),
            Press::default(),
            bindings![KeyCode::ArrowRight, GamepadButton::East],
        ),(
            Action::<AoeInputAction>::new(),
            Press::default(),
            bindings![KeyCode::ArrowDown, GamepadButton::South],
        )]),
    ));
}

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleAudio;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMuted;

#[allow(clippy::needless_pass_by_value)]
fn toggle_audio(
    _toggle_audio: On<Fire<ToggleAudio>>,
    mut audio_sink: Query<&mut AudioSink>,
    mut metronome: ResMut<Metronome>,
) {
    if let Ok(mut audio_sink) = audio_sink.single_mut() {
        if metronome.started {
            audio_sink.pause();
            metronome.started = false;
        } else {
            audio_sink.set_volume(Volume::Linear(1.));
            audio_sink.play();
            metronome.started = true;
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn toggle_muted(
    _toggle_muted: On<Fire<ToggleMuted>>,
    mut audio_sink: Query<&mut AudioSink>,
    mut metronome: ResMut<Metronome>,
) {
    if let Ok(mut audio_sink) = audio_sink.single_mut() {
        if metronome.started {
            audio_sink.pause();
            metronome.started = false;
        } else {
            audio_sink.set_volume(Volume::SILENT);
            audio_sink.play();
            metronome.started = true;
        }
    }
}

#[derive(Component, Debug)]
struct MovementSpeed(f32);

fn player_animation(
    mut player_query: Query<(&Children, &KinematicCharacterController)>,
    mut animation_query: Query<(&mut AseAnimation, &mut Transform)>,
) {
    for (children, kinematic_character_controller) in &mut player_query {
        for child in children.iter() {
            if let Ok((mut ase_sprite_animation, mut transform)) = animation_query.get_mut(child) {
                let velocity = kinematic_character_controller
                    .translation
                    .unwrap_or(Vec2::ZERO);
                let near_idle = velocity.length_squared() < 0.1;
                if near_idle {
                    transform.scale.x = transform.scale.x.abs();
                    ase_sprite_animation.animation.play_loop("idle-right");
                } else if velocity.x > 0. {
                    if velocity.y > 0. {
                        transform.scale.x = -transform.scale.x.abs();
                        ase_sprite_animation.animation.play_loop("walk-up-left");
                    } else {
                        transform.scale.x = transform.scale.x.abs();
                        ase_sprite_animation.animation.play_loop("walk-right");
                    }
                } else if velocity.y > 0. {
                    transform.scale.x = transform.scale.x.abs();
                    ase_sprite_animation.animation.play_loop("walk-up-left");
                } else {
                    transform.scale.x = -transform.scale.x.abs();
                    ase_sprite_animation.animation.play_loop("walk-right");
                }
            }
        }
    }
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[allow(clippy::needless_pass_by_value)]
fn apply_movement(
    movement: On<Fire<Movement>>,
    mut query: Query<
        (&MovementSpeed, &mut KinematicCharacterController),
        (With<Player>, Without<Slide>),
    >,
) {
    if let Ok((movement_speed, mut kinematic_character_controller)) =
        query.get_mut(movement.context)
    {
        kinematic_character_controller.translation =
            Some(movement.value.normalize_or_zero() * movement_speed.0);
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct SlideInputAction;

#[allow(clippy::needless_pass_by_value)]
fn apply_slide(
    slide_input_action: On<Fire<SlideInputAction>>,
    mut commands: Commands,
    query: Query<&KinematicCharacterController>,
    metronome: Res<Metronome>,
    grace_period: Res<GracePeriod>,
) {
    if let Ok(kinematic_character_controller) = query.get(slide_input_action.context)
        && let Some(velocity) = kinematic_character_controller.translation
        && down_beats()
            .iter()
            .any(|&beat| within_nanos_window(&metronome, beat, grace_period.0))
    {
        commands
            .entity(slide_input_action.context)
            .insert(initial_slide(
                10.,
                velocity.normalize_or_zero(),
                1,
                &metronome,
            ));
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct LaserInputAction;

#[allow(clippy::needless_pass_by_value)]
fn apply_laser(
    laser_input_action: On<Fire<LaserInputAction>>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
) {
    #[allow(clippy::cast_possible_truncation)]
    if let Ok(player_transform) = player_query.get(laser_input_action.context)
        && let Some((enemy, _)) = enemy_query
            .iter()
            .sort_by_key::<(Entity, &Transform), i32>(|(_, enemy_transform)| {
                enemy_transform
                    .translation
                    .distance_squared(player_transform.translation) as i32
            })
            .next()
        && down_beats()
            .iter()
            .any(|&beat| within_nanos_window(&metronome, beat, grace_period.0))
    {
        commands
            .entity(laser_input_action.context)
            .with_child(laser_bundle(&laser_sfx, 1, 2, 3., 300., enemy));
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct BulletInputAction;

#[allow(clippy::needless_pass_by_value)]
fn apply_bullet(
    bullet_input_action: On<Fire<BulletInputAction>>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    metronome: Res<Metronome>,
    bullet_sfx: Res<BulletSFX>,
    grace_period: Res<GracePeriod>,
) {
    #[allow(clippy::cast_possible_truncation)]
    if let Ok(player_transform) = player_query.get(bullet_input_action.context)
        && let Some((enemy, _)) = enemy_query
            .iter()
            .sort_by_key::<(Entity, &Transform), i32>(|(_, enemy_transform)| {
                enemy_transform
                    .translation
                    .distance_squared(player_transform.translation) as i32
            })
            .next()
        && down_beats()
            .iter()
            .any(|&beat| within_nanos_window(&metronome, beat, grace_period.0))
    {
        commands.spawn(bullet_bundle(
            &bullet_sfx,
            player_transform,
            3.0,
            150.0,
            3,
            enemy,
        ));
    }
}

#[derive(InputAction)]
#[action_output(bool)]
struct AoeInputAction;

#[allow(clippy::needless_pass_by_value)]
fn apply_aoe(
    aoe_input_action: On<Fire<AoeInputAction>>,
    mut commands: Commands,
    metronome: Res<Metronome>,
    grace_period: Res<GracePeriod>,
) {
    #[allow(clippy::cast_possible_truncation)]
    if down_beats()
        .iter()
        .any(|&beat| within_nanos_window(&metronome, beat, grace_period.0))
    {
        commands
            .entity(aoe_input_action.context)
            .with_child(aoe_bundle(&metronome, 30.0, 75.0, 2));
    }
}

#[allow(clippy::needless_pass_by_value)]
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
fn enemy_movement_system(
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
            commands.entity(entity).insert(initial_slide(
                varied_velocity,
                player_transform.translation.xy() - enemy_transform.translation.xy(),
                1,
                &metronome,
            ));
        }
    }
}
