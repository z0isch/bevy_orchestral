use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{enemy::Enemy, health::Health};

#[derive(Component)]
pub struct Bullet {
    velocity: f32,
    target: Entity,
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
    target: Entity,
) -> BulletBundle {
    BulletBundle {
        bullet: Bullet {
            velocity,
            target,
            damage,
        },
        mesh: Mesh2d(meshes.add(Circle::new(radius))),
        material: MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
        collider: Collider::ball(radius),
        collision_groups: CollisionGroups::new(Group::GROUP_2, Group::ALL),
        sensor: Sensor,
        rigid_body: RigidBody::KinematicVelocityBased,
        active_events: ActiveEvents::COLLISION_EVENTS,
        transform: Transform::from_xyz(
            player_transform.translation.x,
            player_transform.translation.y,
            1.,
        ),
        velocity: Velocity::zero(),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn bullet_system(
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    mut bullet_query: Query<(Entity, &Bullet, &Transform, &mut Velocity)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
) {
    let rapier_context = rapier_context.single().unwrap();

    for (bullet_entity, bullet, transform, mut velocity) in &mut bullet_query {
        if let Ok((enemy_entity, enemy_transform, mut health)) = enemy_query.get_mut(bullet.target)
        {
            let direction =
                (enemy_transform.translation.xy() - transform.translation.xy()).normalize_or_zero();
            velocity.linvel = direction * bullet.velocity;

            if rapier_context.intersection_pair(bullet_entity, enemy_entity) == Some(true) {
                if health.current_health > 0 {
                    health.current_health -= bullet.damage;
                }
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}
