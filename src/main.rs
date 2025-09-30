mod bounce;
mod metronome;
pub mod slide;

use bevy::{
    asset::AssetMetaCheck, input::common_conditions::input_toggle_active, log, prelude::*,
    window::WindowResolution,
};
use bevy_aseprite_ultra::{
    AsepriteUltraPlugin,
    prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation},
};
use bevy_ecs_tilemap::{
    TilemapBundle, TilemapPlugin,
    anchor::TilemapAnchor,
    map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    prelude::{
        ActiveEvents, Collider, CollisionEvent, CollisionGroups, Group,
        KinematicCharacterController, KinematicCharacterControllerOutput, LockedAxes, RigidBody,
        Sensor,
    },
    render::RapierDebugRenderPlugin,
};
use fraction::Fraction;
use rand::{Rng, rng};

use crate::{
    bounce::{Bounce, bounce_system, initial_bounce, initial_tile_bounce, tile_bounce_system},
    metronome::{
        Metronome, down_beats, initial_metronome, is_down_beat, metronome_system, nanos_per_beat,
        within_nanos_window,
    },
    slide::{Slide, initial_slide, slide_system},
};

const WINDOW_WIDTH: f32 = 1024.;
const WINDOW_HEIGHT: f32 = 800.;

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
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        //.add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_systems(Startup, (setup, set_gravity.after(setup)))
        .add_systems(First, metronome_system)
        .add_systems(Update, (update_beat_text, tile_bounce_system))
        .add_systems(Update, toggle_audio)
        .add_systems(
            Update,
            (
                slide_system,
                // Give control back to the player as soon as the slide is done
                control_player,
                bounce_system,
                spawn_enemy_system,
                player_animation,
            )
                .chain(),
        )
        .add_systems(Update, enemy_movement_system)
        .add_systems(Update, display_events)
        .run();
}

fn set_gravity(rapier_config: Query<&mut RapierConfiguration>) {
    let rapier_config = rapier_config.single_inner();
    match rapier_config {
        Ok(mut rapier_config) => rapier_config.gravity = Vec2::ZERO,
        Err(_) => {
            println!("No RapierConfiguration found");
        }
    }
}

#[derive(Component)]
struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(101));
    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    });
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let texture_handle: Handle<Image> = asset_server.load("sprites/kenney_tiny-town/tilemap.png");
    let map_size = TilemapSize {
        x: (WINDOW_WIDTH / tile_size.x / 2.0) as u32,
        y: (WINDOW_HEIGHT / tile_size.y / 2.0) as u32,
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let mut rng = rng();
            let texture_index = if rng.random_range(0..100) < 95 {
                0
            } else if rng.random_range(0..100) < 90 {
                1
            } else {
                2
            };

            let tile = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(texture_index),
                ..Default::default()
            });
            tile_storage.set(&tile_pos, tile.id());
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });

    // Layer 2
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let mut rng = rng();
            let tile_pos = TilePos { x, y };
            let texture_index = if x == 0 && y == 0 {
                Some((68, None)) // bottom-left corner
            } else if x == map_size.x - 1 && y == 0 {
                Some((70, None)) // bottom-right corner
            } else if x == 0 && y == map_size.y - 1 {
                Some((44, None)) // top-left corner
            } else if x == map_size.x - 1 && y == map_size.y - 1 {
                Some((46, None)) // top-right corner
            } else if y == 0 {
                Some((45, None)) // bottom edge
            } else if y == map_size.y - 1 {
                Some((45, None)) // top edge
            } else if x == 0 || x == map_size.x - 1 {
                Some((58, None)) // left or right edge
            } else if rng.random_range(0..100) > 97 {
                Some((29, Some(132)))
            } else {
                None
            };
            match texture_index {
                Some((texture_index, tile_bounce)) => {
                    let tile_pos_in_world = tile_pos.center_in_world(
                        &map_size,
                        &grid_size,
                        &tile_size,
                        &map_type,
                        &TilemapAnchor::Center,
                    );
                    let mut tile = commands.spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(texture_index),
                            ..Default::default()
                        },
                        Transform::from_xyz(tile_pos_in_world.x, tile_pos_in_world.y, 1.),
                        RigidBody::Fixed,
                        Collider::ball(tile_size.x / 2.),
                    ));
                    match tile_bounce {
                        Some(tile_bounce) => {
                            tile.insert(initial_tile_bounce(TileTextureIndex(tile_bounce)));
                        }
                        None => {}
                    }
                    tile_storage.set(&tile_pos, tile.id());
                }
                None => {}
            }
        }
    }
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0., 0., 2.0),
        ..Default::default()
    });

    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>("sounds/song-101bpm.ogg")),
        PlaybackSettings::default().paused(),
    ));
    commands.spawn((
        BeatText,
        Text::new(""),
        TextFont {
            font_size: 100.0,
            ..default()
        },
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
        LockedAxes::ROTATION_LOCKED,
        Collider::round_cuboid(0., 50., 4.75),
        KinematicCharacterController {
            filter_groups: Some(CollisionGroups::new(
                Group::GROUP_1,
                Group::ALL - Group::GROUP_2,
            )),
            ..default()
        },
        MovementSpeed(1.),
        initial_bounce(1.1),
        Player,
        CollisionGroups::new(Group::GROUP_1, Group::ALL - Group::GROUP_2),
    ));
}

#[derive(Component)]
struct BeatText;

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

fn update_beat_text(mut beat_text: Query<&mut Text, With<BeatText>>, metronome: Res<Metronome>) {
    for mut text in beat_text.iter_mut() {
        text.0 = metronome.beat.to_string();
    }
}

#[derive(Component, Debug)]
struct MovementSpeed(f32);

#[derive(Component, Debug)]
struct Player;

fn player_animation(
    mut animation_query: Query<(
        &mut AseAnimation,
        &KinematicCharacterControllerOutput,
        &mut Transform,
    )>,
) {
    for (mut ase_sprite_animation, kinematic_character_controller_output, mut transform) in
        animation_query.iter_mut()
    {
        let velocity = kinematic_character_controller_output.effective_translation;
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
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
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(50.0))),
                MeshMaterial2d(materials.add(Color::hsva(0., 0.95, 0.7, 0.8))),
                Transform::from_xyz(transform.translation.x, transform.translation.y, 1.),
                (RigidBody::Dynamic, Sensor),
                ActiveEvents::COLLISION_EVENTS,
                Collider::ball(50.0),
            ));
        }
        if keyboard_input.just_pressed(KeyCode::KeyK) {
            let grace_period = Fraction::from(90u64 * 1_000_000);

            if down_beats(&metronome)
                .iter()
                .any(|&beat| within_nanos_window(&metronome, beat, grace_period))
            {
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

                commands.entity(entity).insert(initial_slide(
                    transform.translation.xy() + current_direction.normalize() * 150.,
                    nanos_per_beat(metronome.bpm).floor().try_into().unwrap(),
                ));
            }
        }

        let mut translation = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) {
            translation.y = movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            translation.y = -movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            translation.x = -movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            translation.x = movement_speed.0;
        }
        kinematic_character_controller.translation = Some(translation);
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity, entity2, flags) => {
                if let Ok(entity) = enemy_query.get(*entity) {
                    commands.entity(entity).despawn();
                }
                if let Ok(entity2) = enemy_query.get(*entity2) {
                    commands.entity(entity2).despawn();
                }
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
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
                RigidBody::KinematicVelocityBased,
                LockedAxes::ROTATION_LOCKED,
                Collider::ball(45.0 / 2.0),
                KinematicCharacterController::default(),
                MovementSpeed(30.),
                Enemy,
                initial_bounce(1.2),
                CollisionGroups::new(Group::GROUP_2, Group::ALL),
            ));
        }
    }
}

fn enemy_movement_system(
    mut commands: Commands,
    metronome: Res<Metronome>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &MovementSpeed), With<Enemy>>,
) {
    if metronome.started && metronome.is_beat_start_frame && is_down_beat(&metronome) {
        if let Ok(player_transform) = player_query.single() {
            for (entity, enemy_transform, movement_speed) in enemy_query.iter_mut() {
                let mut rng = rng();
                let speed_variation = rng.random_range(-0.2..=0.2);
                let varied_velocity = movement_speed.0 * (1.0 + speed_variation);
                let direction = (player_transform.translation.xy()
                    - enemy_transform.translation.xy())
                .normalize();

                commands.entity(entity).insert(initial_slide(
                    enemy_transform.translation.xy() + direction * varied_velocity,
                    nanos_per_beat(metronome.bpm).floor().try_into().unwrap(),
                ));
            }
        }
    }
}
