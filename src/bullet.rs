use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{enemy::Enemy, health::Health, nearest_enemy::NearestEnemy};

#[derive(Component)]
pub struct Bullet {
    velocity: f32,
    damage: u128,
    target: Option<Entity>,
}

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    mesh: Mesh2d,
    sensor: Sensor,
    material: MeshMaterial2d<ColorMaterial>,
    collider: Collider,
    collision_groups: CollisionGroups,
    transform: Transform,
    velocity: Velocity,
    rigid_body: RigidBody,
    nearest_enemy: NearestEnemy,
}

pub fn bullet_bundle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    player_transform: &Transform,
    radius: f32,
    velocity: f32,
    damage: u128,
) -> BulletBundle {
    BulletBundle {
        bullet: Bullet {
            velocity,
            damage,
            target: None,
        },
        mesh: Mesh2d(meshes.add(Circle::new(radius))),
        material: MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
        collider: Collider::ball(radius),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        rigid_body: RigidBody::KinematicVelocityBased,
        transform: Transform::from_xyz(
            player_transform.translation.x,
            player_transform.translation.y,
            1.,
        ),
        velocity: Velocity::zero(),
        nearest_enemy: NearestEnemy(None),
    }
}

pub fn bullet_system(
    mut commands: Commands,
    mut bullet_query: Query<(
        Entity,
        &mut Bullet,
        &Transform,
        &mut Velocity,
        &NearestEnemy,
    )>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, mut bullet, transform, mut velocity, nearest_enemy) in &mut bullet_query {
        if bullet.target.is_none()
            && let Some(nearest_enemy) = nearest_enemy.0
        {
            bullet.target = Some(nearest_enemy);
        }

        if let Some(target) = bullet.target
            && let Ok((_, enemy_transform)) = enemy_query.get(target)
        {
            let direction =
                (enemy_transform.translation.xy() - transform.translation.xy()).normalize_or_zero();
            velocity.linvel = direction * bullet.velocity;
        } else {
            commands.entity(bullet_entity).despawn();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn bullet_collision_system(
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    mut bullet_query: Query<(Entity, &Bullet)>,
    mut enemy_query: Query<(Entity, &mut Health), With<Enemy>>,
) {
    for (bullet_entity, bullet) in &mut bullet_query {
        let rapier_context = rapier_context.single().unwrap();
        for (enemy_entity, mut health) in &mut enemy_query {
            if rapier_context.intersection_pair(bullet_entity, enemy_entity) == Some(true) {
                if health.current_health > 0 {
                    health.current_health -= bullet.damage;
                }
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}
