#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]
mod aoe;
mod bounce;
mod bullet;
mod enemy;
mod follower;
mod health;
mod instrument;
mod laser;
mod map;
mod metronome;
mod nearest_entity;
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
use bevy_hotpatching_experiments::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    prelude::{
        ActiveCollisionTypes, Collider, KinematicCharacterController, LockedAxes, QueryFilterFlags,
        RigidBody, Velocity,
    },
};
use fraction::Fraction;

use crate::{
    aoe::{aoe_bundle, aoe_collision_system, aoe_system, process_aoe_duration},
    bounce::{bounce_system, initial_bounce, tile_bounce_system},
    bullet::{
        bullet_collision_system, bullet_launcher_bundle, bullet_launcher_system, bullet_system,
        setup_bullet_sfx,
    },
    enemy::{
        Enemy, EnemySpawnTimer, raccoon_bullet_collision_system, raccoon_bullet_system,
        raccoon_movement_system, skunk_movement_system, spawn_raccoon_system, spawn_skunk_system,
    },
    follower::follower_system,
    health::{despawn_enemy_on_zero_health, health_bar_system, on_health_bar_add},
    instrument::{Violin, spawn_violin},
    laser::{LaserSFX, laser_bundle, laser_system, setup_laser_sfx},
    map::setup_map,
    metronome::{Metronome, down_beats, initial_metronome, metronome_system, within_nanos_window},
    note_highway::{
        beat_line_system, note_highway_system, on_beat_line_system, setup_note_highway,
    },
    player::Player,
    slide::{Slide, initial_slide, slide_system},
    window_size::{WINDOW_HEIGHT, WINDOW_WIDTH, setup_window_size},
};

const SONG_BPM: u64 = 85;
const SONG_FILE: &str = "sounds/clicktrack-85bpm.ogg";

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
        .add_plugins(SimpleSubsecondPlugin::default())
        .add_input_context::<Player>()
        .add_input_context::<Song>()
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
        .add_systems(First, metronome_system)
        .add_systems(Update, (destroy_all_enemies, spawn_new_violin))
        .add_systems(Update, follower_system)
        .add_systems(
            Update,
            (
                spawn_raccoon_system,
                raccoon_movement_system,
                raccoon_bullet_collision_system,
                raccoon_bullet_system,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_skunk_system,
                spawn_skunk_system,
                skunk_movement_system,
            ),
        )
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
                bullet_launcher_system,
                laser_system,
                despawn_enemy_on_zero_health,
                health_bar_system,
                bullet_collision_system,
                slide_system,
                player_animation,
            ),
        )
        .add_observer(on_health_bar_add)
        .add_observer(apply_movement)
        .add_observer(toggle_audio)
        .add_observer(toggle_muted)
        .add_observer(apply_slide)
        .add_observer(apply_north_note_played)
        .add_observer(apply_east_note_played)
        .add_observer(apply_south_note_played)
        .add_observer(apply_west_note_played)
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

#[allow(clippy::needless_pass_by_value)]
fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(SONG_BPM));
    commands.insert_resource(GracePeriod(Fraction::from(90u64 * 1_000_000)));
    commands.insert_resource(EnemySpawnTimer {
        skunk_timer: Timer::from_seconds(2., TimerMode::Repeating),
        raccoon_timer: Timer::from_seconds(3.5, TimerMode::Repeating),
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
        Song,
        actions!(Song[(
            Action::<ToggleAudio>::new(),
            Press::default(),
            bindings![KeyCode::KeyX, GamepadButton::Start],
        ),(
            Action::<ToggleMuted>::new(),
            Press::default(),
            bindings![KeyCode::KeyZ, GamepadButton::Select],
        )]),
    ));

    let player_sprite_scale = 0.15;
    let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
    sprite_transform.scale = Vec3::new(player_sprite_scale, player_sprite_scale, 0.);
    commands.spawn((
        Transform::from_xyz(0., 0., 2.),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::ONLY_FIXED,
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule_y(100. * player_sprite_scale, 25. * player_sprite_scale),
        ActiveCollisionTypes::KINEMATIC_KINEMATIC,
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
            Action::<SlideInputAction>::new(),
            Press::default(),
            bindings![KeyCode::Space, GamepadButton::LeftTrigger],
        ),(
            Action::<NorthNotePlayed>::new(),
            Press::default(),
            bindings![KeyCode::ArrowUp, GamepadButton::North],
        ),(
            Action::<EastNotePlayed>::new(),
            Press::default(),
            bindings![KeyCode::ArrowRight, GamepadButton::East],
        ),(
            Action::<SouthNotePlayed>::new(),
            Press::default(),
            bindings![KeyCode::ArrowDown, GamepadButton::South],
        ),
        (
            Action::<WestNotePlayed>::new(),
            Press::default(),
            bindings![KeyCode::ArrowLeft, GamepadButton::West],
        )]),
    ));
}

#[derive(Component)]
struct Song;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleAudio;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMuted;

#[allow(clippy::needless_pass_by_value)]
fn toggle_audio(
    _toggle_audio: On<Fire<ToggleAudio>>,
    mut audio_sink: Query<&mut AudioSink, With<Song>>,
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
    mut audio_sink: Query<&mut AudioSink, With<Song>>,
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

#[allow(clippy::needless_pass_by_value)]
fn destroy_all_enemies(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    if input.just_pressed(KeyCode::KeyC) {
        for enemy_entity in enemy_query.iter() {
            commands.entity(enemy_entity).try_despawn();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_new_violin(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    violin_query: Query<(Entity, &Transform), With<Violin>>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        if violin_query.is_empty()
            && let Ok((player_entity, player_transform)) = player_query.single()
        {
            spawn_violin(
                &mut commands,
                asset_server,
                player_entity,
                30.,
                player_transform.translation.xy(),
            );
        } else {
            let last_violin = violin_query.iter().last();
            if let Some((last_violin_entity, last_violin_transform)) = last_violin {
                spawn_violin(
                    &mut commands,
                    asset_server,
                    last_violin_entity,
                    30.,
                    last_violin_transform.translation.xy(),
                );
            }
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

pub enum NotePlayed {
    NorthNote,
    EastNote,
    SouthNote,
    WestNote,
}

#[derive(InputAction)]
#[action_output(bool)]
struct NorthNotePlayed();

#[derive(InputAction)]
#[action_output(bool)]
struct EastNotePlayed();

#[derive(InputAction)]
#[action_output(bool)]
struct SouthNotePlayed();

#[derive(InputAction)]
#[action_output(bool)]
struct WestNotePlayed();

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn apply_north_note_played(
    note_played: On<Fire<NorthNotePlayed>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    commands: Commands,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
    violin_query: Query<Entity, With<Violin>>,
) {
    apply_note_played(
        meshes,
        materials,
        NotePlayed::NorthNote,
        note_played.context,
        commands,
        metronome,
        laser_sfx,
        grace_period,
        violin_query,
    );
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn apply_east_note_played(
    note_played: On<Fire<EastNotePlayed>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    commands: Commands,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
    violin_query: Query<Entity, With<Violin>>,
) {
    apply_note_played(
        meshes,
        materials,
        NotePlayed::EastNote,
        note_played.context,
        commands,
        metronome,
        laser_sfx,
        grace_period,
        violin_query,
    );
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn apply_south_note_played(
    note_played: On<Fire<SouthNotePlayed>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    commands: Commands,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
    violin_query: Query<Entity, With<Violin>>,
) {
    apply_note_played(
        meshes,
        materials,
        NotePlayed::SouthNote,
        note_played.context,
        commands,
        metronome,
        laser_sfx,
        grace_period,
        violin_query,
    );
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn apply_west_note_played(
    note_played: On<Fire<WestNotePlayed>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    commands: Commands,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
    violin_query: Query<Entity, With<Violin>>,
) {
    apply_note_played(
        meshes,
        materials,
        NotePlayed::WestNote,
        note_played.context,
        commands,
        metronome,
        laser_sfx,
        grace_period,
        violin_query,
    );
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn apply_note_played(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    note_played: NotePlayed,
    player_entity: Entity,
    mut commands: Commands,
    metronome: Res<Metronome>,
    laser_sfx: Res<LaserSFX>,
    grace_period: Res<GracePeriod>,
    violin_query: Query<Entity, With<Violin>>,
) {
    if down_beats()
        .iter()
        .any(|&beat| within_nanos_window(&metronome, beat, grace_period.0))
    {
        match note_played {
            NotePlayed::NorthNote => {
                for violin_entity in violin_query.iter() {
                    commands.spawn(laser_bundle(
                        &mut meshes,
                        &mut materials,
                        &laser_sfx,
                        1,
                        4,
                        10.,
                        500.,
                        violin_entity,
                    ));
                }
            }
            NotePlayed::EastNote => {
                commands
                    .entity(player_entity)
                    .with_child(bullet_launcher_bundle(3.0, 150.0, 2, 12));
            }
            NotePlayed::SouthNote => {
                commands
                    .entity(player_entity)
                    .with_child(aoe_bundle(&metronome, 30.0, 75.0, 2));
            }
            NotePlayed::WestNote => {}
        }
    }
}
