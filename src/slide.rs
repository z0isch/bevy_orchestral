use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{KinematicCharacterController, Velocity};

use crate::{
    aoe::AoeDuration,
    metronome::{Metronome, nanos_per_beat},
};

#[derive(Component)]
pub struct Slide {
    velocity: f32,
    direction: Vec2,
    timer: Timer,
}

pub fn initial_slide(
    velocity: f32,
    direction: Vec2,
    num_beats_duration: u8,
    metronome: &Metronome,
) -> Slide {
    Slide {
        velocity,
        direction,
        timer: Timer::new(
            Duration::from_nanos(nanos_per_beat(metronome.bpm) * u64::from(num_beats_duration)),
            TimerMode::Repeating,
        ),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn slide_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Option<&mut KinematicCharacterController>,
            Option<&mut Velocity>,
            &mut Slide,
        ),
        Without<AoeDuration>,
    >,
) {
    for (entity, kinematic_character_controller, velocity, mut slide) in &mut query {
        slide.timer.tick(time.delta());
        if slide.timer.just_finished() {
            commands.entity(entity).try_remove::<Slide>();
            if let Some(mut velocity) = velocity {
                velocity.linvel = Vec2::ZERO;
            }
        } else if let Some(mut kinematic_character_controller) = kinematic_character_controller {
            kinematic_character_controller.translation =
                Some(slide.direction.normalize_or_zero() * slide.velocity);
        } else if let Some(mut velocity) = velocity {
            velocity.linvel = slide.direction.normalize_or_zero() * slide.velocity * 10.;
        }
    }
}
