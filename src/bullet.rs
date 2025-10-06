use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{enemy::Enemy, health::Health};

#[derive(Component)]
pub struct Bullet {
    velocity: f32,
    enemy: Entity,
    damage: u128,
}

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    mesh: Mesh2d,
    active_events: ActiveEvents,
    sensor: Sensor,
    material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    transform: Transform,
    velocity: Velocity,
    rigid_body: RigidBody,
}

pub fn bullet_bundle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
    radius: f32,
    velocity: f32,
    damage: u128,
    enemy: Entity,
) -> BulletBundle {
    BulletBundle {
        bullet: Bullet {
            velocity,
            enemy,
            damage,
        },
        mesh: Mesh2d(meshes.add(Circle::new(radius))),
        material: MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
        collider: Collider::ball(radius),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        rigid_body: RigidBody::Dynamic,
        active_events: ActiveEvents::COLLISION_EVENTS,
        transform: Transform::from_xyz(
            player_transform.translation.x,
            player_transform.translation.y,
            1.,
        ),
        velocity: Velocity::zero(),
    }
}

pub fn bullet_system(
    mut bullet_query: Query<(&Bullet, &Transform, &mut Velocity)>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (bullet, transform, mut velocity) in bullet_query.iter_mut() {
        if let Ok(enemy_transform) = enemy_query.get(bullet.enemy) {
            let direction =
                (enemy_transform.translation.xy() - transform.translation.xy()).normalize_or_zero();
            velocity.linvel = direction * bullet.velocity;
        }
    }
}

pub fn bullet_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    query_bullet: Query<(Entity, &Bullet, &Transform)>,
    mut query_enemy: Query<(Entity, &mut Health, &Transform), With<Enemy>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let (bullet_entity, enemy_entity) =
                if query_bullet.get(*entity1).is_ok() && query_enemy.get(*entity2).is_ok() {
                    (*entity1, *entity2)
                } else if query_bullet.get(*entity2).is_ok() && query_enemy.get(*entity1).is_ok() {
                    (*entity2, *entity1)
                } else {
                    continue;
                };
            if let Ok((_, bullet, _)) = query_bullet.get(bullet_entity) {
                if let Ok((_, mut health, _)) = query_enemy.get_mut(enemy_entity) {
                    health.current_health -= bullet.damage;
                }
            }
            commands.entity(bullet_entity).despawn();
        }
    }
}
