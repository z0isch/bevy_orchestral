use bevy::prelude::*;
use bevy_rapier2d::prelude::KinematicCharacterController;

#[derive(Component, Debug)]
pub struct Slide {
    start_position: Option<Vec2>,
    end_position: Vec2,
    nanos_duration: u128,
    nanos_elapsed: Option<u128>,
}

pub fn initial_slide(end_position: Vec2, nanos_duration: u128) -> Slide {
    Slide {
        start_position: None,
        end_position,
        nanos_duration,
        nanos_elapsed: None,
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
        let start_position = *slide
            .start_position
            .get_or_insert(transform.translation.xy());
        let nanos_elapsed = *slide.nanos_elapsed.get_or_insert(0);

        let direction = slide.end_position - start_position;
        let current_offset = transform.translation.xy() - start_position;
        let projection = current_offset.dot(direction.normalize());

        if nanos_elapsed >= slide.nanos_duration || projection >= direction.length() {
            commands.entity(entity).remove::<Slide>();
        } else {
            let velocity_needed = direction / (slide.nanos_duration as f32 / 1_000_000_000.0);
            kinematic_character_controller.translation = Some(velocity_needed / 100.);
        }
        slide.nanos_elapsed = Some(nanos_elapsed + time.delta().as_nanos());
    }
}
