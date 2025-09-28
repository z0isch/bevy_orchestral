use avian2d::{
    PhysicsPlugins,
    prelude::{LinearVelocity, RigidBody},
};
use bevy::prelude::*;
use bevy_aseprite_ultra::{
    AsepriteUltraPlugin,
    prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation},
};
use fraction::Fraction;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(AsepriteUltraPlugin)
        .add_systems(Startup, setup)
        .add_systems(First, process_metronome)
        .add_systems(Update, (update_beat_text, bounce_player))
        .add_systems(Update, toggle_audio)
        .add_systems(Update, (control_player, player_animation))
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(initial_metronome(101));
    commands.spawn(Camera2d);
    commands.spawn((
        AudioPlayer::new(asset_server.load::<AudioSource>("sounds/clicktrack-101bpm.mp3")),
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
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 0.));
    transform.scale = Vec3::new(0.5, 0.5, 0.);
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
        Bounce,
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

fn process_metronome(time: Res<Time>, mut metronome: ResMut<Metronome>) {
    if metronome.started {
        let nanos_per_beat = nanos_per_beat(metronome.bpm);
        metronome.nanos_accumulated += Fraction::from(time.delta().as_nanos());
        if metronome.nanos_accumulated >= nanos_per_beat {
            metronome.is_beat_start_frame = true;
            metronome.beat = (metronome.beat + 1) % 16;
            metronome.nanos_accumulated -= nanos_per_beat;
        } else {
            metronome.is_beat_start_frame = false;
        }
    }
}

fn update_beat_text(mut beat_text: Query<&mut Text, With<BeatText>>, metronome: Res<Metronome>) {
    for mut text in beat_text.iter_mut() {
        text.0 = metronome.beat.to_string();
    }
}

#[derive(Component)]
struct Bounce;

fn bounce_player(metronome: Res<Metronome>, mut bouncers: Query<&mut Transform, With<Bounce>>) {
    for mut transform in bouncers.iter_mut() {
        if metronome.is_beat_start_frame {
            if metronome.beat == 1 {
                transform.scale.y *= 1.1;
            }
            if metronome.beat == 3 {
                transform.scale.y /= 1.1;
            }
        }
    }
}

#[derive(Resource)]
struct Metronome {
    beat: u8,
    bpm: u64,
    is_beat_start_frame: bool,
    nanos_accumulated: Fraction,
    started: bool,
}

fn initial_metronome(bpm: u64) -> Metronome {
    return Metronome {
        beat: 0,
        bpm: bpm,
        is_beat_start_frame: false,
        nanos_accumulated: Fraction::from(0),
        started: false,
    };
}

fn nanos_per_beat(bpm: u64) -> Fraction {
    return Fraction::new(60_000_000_000u64, bpm * 4);
}

fn nanos_distance_to_beat(beat: u8, metronome: &Metronome) -> Fraction {
    let shifted = metronome.beat as i8 - beat as i8;
    let on_beat_or_after_beat = shifted >= 0 && shifted <= 8;
    let since_start_or_till_next = if on_beat_or_after_beat {
        metronome.nanos_accumulated
    } else {
        nanos_per_beat(metronome.bpm) - metronome.nanos_accumulated
    };
    let beat_distance = 8 - ((beat as i8 - metronome.beat as i8).abs() - 8).abs();
    let factor = 0.max(beat_distance - 1);
    let mut base = since_start_or_till_next;
    for _i in 0..factor {
        base = base + nanos_per_beat(metronome.bpm);
    }
    return base;
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
    mut query: Query<(&mut LinearVelocity, &MovementSpeed, &mut Transform), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut velocity, movement_speed, mut transform) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            let grace_period = Fraction::from(60u64 * 1_000_000);
            let nanos_distance = nanos_distance_to_beat(0, &metronome);
            println!(
                "nanos_distance_to_beat: {}, grace_period: {}",
                (nanos_distance.numer().unwrap() / nanos_distance.denom().unwrap()) as f64
                    / 1_000_000.,
                (grace_period.numer().unwrap() / grace_period.denom().unwrap()) as f64 / 1_000_000.,
            );
            if nanos_distance <= grace_period {
                transform.translation =
                    transform.translation + Vec3::new(velocity.x, velocity.y, 0.);
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
