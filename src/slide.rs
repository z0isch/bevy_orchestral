use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{ExternalImpulse, KinematicCharacterController, Velocity};

use crate::metronome::{Metronome, nanos_per_beat};

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
            Duration::from_nanos(nanos_per_beat(metronome.bpm) * num_beats_duration as u64),
            TimerMode::Repeating,
        ),
    }
}

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
        Without<ExternalImpulse>,
    >,
) {
    for (entity, kinematic_character_controller, velocity, mut slide) in query.iter_mut() {
        slide.timer.tick(time.delta());
        if slide.timer.just_finished() {
            commands.entity(entity).remove::<Slide>();
            if let Some(mut velocity) = velocity {
                velocity.linvel = Vec2::ZERO;
            }
        } else {
            if let Some(mut kinematic_character_controller) = kinematic_character_controller {
                kinematic_character_controller.translation =
                    Some(slide.direction.normalize_or_zero() * slide.velocity);
            } else if let Some(mut velocity) = velocity {
                velocity.linvel = slide.direction.normalize_or_zero() * slide.velocity * 10.;
            }
        }
    }
}
