use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::KinematicCharacterController;

#[derive(Component, Debug)]
pub struct Slide {
    start_position: Option<Vec2>,
    end_position: Vec2,
    timer: Timer,
}

pub fn initial_slide(end_position: Vec2, nanos_duration: u64) -> Slide {
    Slide {
        start_position: None,
        end_position,
        timer: Timer::new(Duration::from_nanos(nanos_duration), TimerMode::Once),
    }
}

pub fn slide_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Transform,
        &mut KinematicCharacterController,
        &mut Slide,
    )>,
    mut commands: Commands,
) {
    for (entity, transform, mut kinematic_character_controller, mut slide) in query.iter_mut() {
        slide.timer.tick(time.delta());
        if slide.timer.just_finished() {
            commands.entity(entity).remove::<Slide>();
        } else {
            let start_position = *slide
                .start_position
                .get_or_insert(transform.translation.xy());

            let direction = slide.end_position - start_position;
            let current_offset = transform.translation.xy() - start_position;
            let projection = current_offset.dot(direction.normalize());

            if projection >= direction.length() {
                commands.entity(entity).remove::<Slide>();
            } else {
                let velocity_needed =
                    direction / (slide.timer.duration().as_nanos() as f32 / 1_000_000_000.0);
                kinematic_character_controller.translation = Some(velocity_needed / 100.);
            }
        }
    }
}
