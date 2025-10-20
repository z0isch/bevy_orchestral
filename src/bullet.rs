use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::Enemy,
    health::Health,
    metronome::{Metronome, MetronomeTimer},
    nearest_entity::find_nearest_entity,
};

#[derive(Component)]
pub struct BulletLauncher {
    radius: f32,
    velocity: f32,
    damage: u128,
    timer: MetronomeTimer,
    last_fired_on_beat: Option<u8>,
}

#[derive(Component)]
pub struct Bullet {
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
pub struct BulletLauncherBundle {
    bullet_launcher: BulletLauncher,
    transform: Transform,
}

pub fn bullet_launcher_bundle(
    radius: f32,
    velocity: f32,
    damage: u128,
    number_beats_duration: u8,
) -> BulletLauncherBundle {
    BulletLauncherBundle {
        bullet_launcher: BulletLauncher {
            radius,
            velocity,
            timer: MetronomeTimer::new(number_beats_duration),
            damage,
            last_fired_on_beat: None,
        },

        transform: Transform::from_xyz(0., 0., 2.),
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)]
pub fn bullet_launcher_system(
    bullet_sfx: Res<BulletSFX>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    metronome: Res<Metronome>,
    time: Res<Time>,
    mut bullet_launcher_query: Query<(Entity, &mut BulletLauncher, &ChildOf)>,
    parent_query: Query<&Transform, (With<Children>, (Without<Enemy>, Without<BulletLauncher>))>,
) {
    for (bullet_launcher_entity, mut bullet_launcher, parent) in &mut bullet_launcher_query {
        if let Ok(parent_transform) = parent_query.get(parent.parent()) {
            bullet_launcher.timer.tick(&metronome, *time);
            if bullet_launcher.timer.just_finished(&metronome) {
                commands.entity(bullet_launcher_entity).try_despawn();
            } else if !bullet_launcher.timer.finished()
                && bullet_launcher.last_fired_on_beat.map_or_else(
                    || true,
                    |b| metronome.is_beat_start_frame && b != bullet_launcher.timer.beats_elapsed(),
                )
            {
                bullet_launcher.last_fired_on_beat = Some(bullet_launcher.timer.beats_elapsed());
                commands.spawn((
                    Bullet {
                        velocity: bullet_launcher.velocity,
                        damage: bullet_launcher.damage,
                        target: None,
                    },
                    Transform::from_xyz(
                        parent_transform.translation.x,
                        parent_transform.translation.y,
                        2.,
                    ),
                    Velocity::zero(),
                    AudioPlayer::new(bullet_sfx.fire.clone()),
                    Mesh2d(meshes.add(Circle::new(bullet_launcher.radius))),
                    MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
                    Collider::ball(bullet_launcher.radius),
                    CollisionGroups::new(Group::GROUP_2, Group::ALL),
                    Sensor,
                    RigidBody::KinematicVelocityBased,
                ));
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn bullet_system(
    mut bullet_query: Query<(&mut Bullet, &mut Velocity, &Transform)>,
    mut enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (mut bullet, mut velocity, transform) in &mut bullet_query {
        if bullet
            .target
            .map_or_else(|| true, |target| enemy_query.get(target).is_err())
            && let Some((enemy_entity, _)) =
                find_nearest_entity(*transform, enemy_query.transmute_lens().query())
        {
            bullet.target = Some(enemy_entity);
        }

        if let Some(target) = bullet.target
            && let Ok((_, enemy_transform)) = enemy_query.get(target)
        {
            let direction =
                (enemy_transform.translation.xy() - transform.translation.xy()).normalize_or_zero();
            velocity.linvel = direction * bullet.velocity;
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
