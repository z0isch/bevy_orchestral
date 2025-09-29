use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Slide {
    start_position: Option<Vec3>,
    end_position: Vec3,
    nanos_duration: u128,
}

pub fn initial_slide(end_position: Vec3, nanos_duration: u128) -> Slide {
    Slide {
        start_position: None,
        end_position,
        nanos_duration,
    }
}

pub fn slide_system(
    mut query: Query<(Entity, &Transform, &mut LinearVelocity, &mut Slide)>,
    mut commands: Commands,
) {
    for (entity, transform, mut velocity, mut slide) in query.iter_mut() {
        let start_position = *slide.start_position.get_or_insert(transform.translation);

        let direction = slide.end_position - start_position;
        let current_offset = transform.translation - start_position;
        let projection = current_offset.dot(direction.normalize());

        if projection >= direction.length() {
            velocity.x = 0.;
            velocity.y = 0.;
            commands.entity(entity).remove::<Slide>();
        } else {
            let velocity_needed = direction / (slide.nanos_duration as f32 / 1_000_000_000.0);
            velocity.x = velocity_needed.x;
            velocity.y = velocity_needed.y;
        }
    }
}
