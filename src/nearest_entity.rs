use bevy::prelude::*;

pub fn find_nearest_entity(
    subject_transform: Transform,
    query: Query<(Entity, &Transform)>,
) -> Option<(Entity, Transform)> {
    #[allow(clippy::cast_possible_truncation)]
    query
        .iter()
        .sort_by_key::<(Entity, &Transform), i32>(|(_, enemy_transform)| {
            enemy_transform
                .translation
                .distance_squared(subject_transform.translation) as i32
        })
        .next()
        .map(|(entity, transform)| (entity, *transform))
}
