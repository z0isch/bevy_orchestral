use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Component)]
pub struct Follower {
    pub following: Entity,
    pub follow_distance: f32,
}

pub fn follower_system(
    mut transforms: ParamSet<(
        Query<(Entity, &mut Transform, &Follower)>,
        Query<&Transform>,
    )>,
) {
    let follower_data: Vec<_> = transforms
        .p0()
        .iter()
        .map(|(entity, transform, follower)| {
            (
                entity,
                transform.translation,
                follower.following,
                follower.follow_distance,
            )
        })
        .collect();

    let mut updates = HashMap::new();
    for (follower_entity, follower_translation, following_entity, follow_distance) in follower_data
    {
        if let Ok(following_transform) = transforms.p1().get(following_entity) {
            let distance_between = follower_translation.distance(following_transform.translation);
            let distance_to_stay_in_range = distance_between - follow_distance;
            if distance_to_stay_in_range > 0. {
                let direction =
                    (following_transform.translation - follower_translation).normalize_or_zero();
                let new_translation = follower_translation + direction * distance_to_stay_in_range;
                updates.insert(follower_entity, new_translation);
            }
        }
    }

    for (entity, mut follower_transform, _) in &mut transforms.p0() {
        if let Some(new_translation) = updates.get(&entity) {
            follower_transform.translation = *new_translation;
        }
    }
}
