use bevy::{ecs::prelude::*, transform::components::Transform};

use crate::enemy::Enemy;

#[derive(Component)]
pub struct NearestEnemy(pub Option<Entity>);

pub fn target_nearest_enemy(
    mut nearest_enemy_query: Query<(Option<&ChildOf>, &mut NearestEnemy, &Transform)>,
    parent_query: Query<&Transform, With<Children>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (m_parent, mut nearest_enemy, nearest_enemy_transform) in &mut nearest_enemy_query {
        let nearest_enemy_transform = if let Some(parent) = m_parent
            && let Ok(parent_transform) = parent_query.get(parent.parent())
        {
            parent_transform
        } else {
            nearest_enemy_transform
        };

        #[allow(clippy::cast_possible_truncation)]
        match enemy_query
            .iter()
            .sort_by_key::<(Entity, &Transform), i32>(|(_, enemy_transform)| {
                enemy_transform
                    .translation
                    .distance_squared(nearest_enemy_transform.translation) as i32
            })
            .next()
        {
            Some((enemy, _)) => {
                nearest_enemy.0 = Some(enemy);
            }
            None => {
                nearest_enemy.0 = None;
            }
        }
    }
}
