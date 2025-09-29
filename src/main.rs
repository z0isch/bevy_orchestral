mod bounce;
mod metronome;
pub mod slide;

use avian2d::{
    PhysicsPlugins,
    prelude::{LinearVelocity, RigidBody},
};
use bevy::{asset::AssetMetaCheck, prelude::*};
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
use fraction::Fraction;
use rand::{Rng, rng};

use crate::{
    bounce::{Bounce, bounce_system},
    metronome::{Metronome, initial_metronome, metronome_system, within_nanos_window},
    slide::{Slide, initial_slide, slide_system},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(AsepriteUltraPlugin)
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .add_systems(First, metronome_system)
        .add_systems(Update, (update_beat_text, bounce_system))
        .add_systems(Update, toggle_audio)
        .add_systems(
            Update,
            (
                slide_system,
                // Give control back to the player as soon as the slide is done
                control_player.after(slide_system),
                player_animation.after(control_player),
            ),
        )
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(101));
    commands.spawn(Camera2d);
    let texture_handle: Handle<Image> =
        asset_server.load("sprites/kenney_tiny-town/tilemap.png");

    let map_size = TilemapSize { x: 100, y: 100 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let mut rng = rng();
            let random_texture_index = if rng.random_range(0..100) < 95 {
                0
            } else {
                if rng.random_range(0..100) < 90 { 1 } else { 2 }
            };
            let tile = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(random_texture_index),
                ..Default::default()
            });
            tile_storage.set(&tile_pos, tile.id());
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
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
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::new(0.3, 0.3, 1.);
    commands.spawn((
        AseAnimation {
            animation: Animation::tag("idle-right")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.5),
            aseprite: asset_server.load("sprites/maestro.aseprite"),
        },
        Sprite::default(),
        transform,
        RigidBody::Dynamic,
        LinearVelocity(Vec2::new(0., 0.)),
        MovementSpeed(150.),
        Bounce { scale: 1.1 },
        Player,
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
    mut animation_query: Query<(&mut AseAnimation, &LinearVelocity, &mut Transform), With<Player>>,
) {
    for (mut ase_sprite_animation, velocity, mut transform) in animation_query.iter_mut() {
        if velocity.x == 0. && velocity.y == 0. {
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
    mut query: Query<
        (Entity, &mut LinearVelocity, &MovementSpeed, &Transform),
        (With<Player>, Without<Slide>),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (entity, mut velocity, movement_speed, transform) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            let grace_period = Fraction::from(90u64 * 1_000_000);
            if within_nanos_window(&metronome, 0, grace_period) {
                commands.entity(entity).insert(initial_slide(
                    transform.translation
                        + Vec3::new(velocity.x, velocity.y, 0.).normalize() * 150.,
                    200u128 * 1_000_000,
                ));
            }
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity.y = movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            velocity.y = -movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity.x = -movement_speed.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity.x = movement_speed.0;
        }

        if keyboard_input.just_released(KeyCode::KeyW)
            || keyboard_input.just_released(KeyCode::KeyS)
        {
            velocity.y = 0.;
        }
        if keyboard_input.just_released(KeyCode::KeyA)
            || keyboard_input.just_released(KeyCode::KeyD)
        {
            velocity.x = 0.;
        }
    }
}
