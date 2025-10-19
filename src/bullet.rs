use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{enemy::Enemy, health::Health, nearest_entity::find_nearest_entity};

#[derive(Component)]
pub struct Bullet {
    radius: f32,
    velocity: f32,
    damage: u128,
    target: Option<Entity>,
}

#[derive(Resource)]
pub struct BulletSFX {
    pub fire: Handle<AudioSource>,
}

#[allow(clippy::needless_pass_by_value)]
pub fn setup_bullet_sfx(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(BulletSFX {
        fire: asset_server.load("sounds/bullet.ogg"),
    });
}

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    transform: Transform,
    velocity: Velocity,
    audio_player: AudioPlayer,
}

pub fn bullet_bundle(
    bullet_sfx: &Res<BulletSFX>,
    radius: f32,
    velocity: f32,
    damage: u128,
) -> BulletBundle {
    BulletBundle {
        bullet: Bullet {
            radius,
            velocity,
            damage,
            target: None,
        },

        transform: Transform::from_xyz(0., 0., 2.),
        velocity: Velocity::zero(),
        audio_player: AudioPlayer::new(bullet_sfx.fire.clone()),
    }
}

pub fn bullet_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Bullet, &mut Velocity, &ChildOf)>,
    parent_query: Query<&Transform, (With<Children>, (Without<Enemy>, Without<Bullet>))>,
    mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, mut bullet, mut velocity, parent) in &mut bullet_query {
        if let Ok(parent_transform) = parent_query.get(parent.parent()) {
            if bullet.target.is_none()
                && let Some((enemy_entity, _)) =
                    find_nearest_entity(*parent_transform, enemy_query.transmute_lens().query())
            {
                bullet.target = Some(enemy_entity);
            }

            if let Some(target) = bullet.target
                && let Ok((_, enemy_transform)) = enemy_query.get(target)
            {
                let direction = (enemy_transform.translation.xy()
                    - parent_transform.translation.xy())
                .normalize_or_zero();
                velocity.linvel = direction * bullet.velocity;
                commands.entity(bullet_entity).try_insert_if_new((
                    Mesh2d(meshes.add(Circle::new(bullet.radius))),
                    MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
                    Collider::ball(bullet.radius),
                    CollisionGroups::new(Group::GROUP_2, Group::ALL),
                    Sensor,
                    RigidBody::KinematicVelocityBased,
                ));
            } else {
                commands.entity(bullet_entity).try_despawn();
            }
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
                health.current_health = health.current_health.saturating_sub(bullet.damage);
                commands.entity(bullet_entity).try_despawn();
            }
        }
    }
}
