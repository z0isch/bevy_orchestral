use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RigidBody};

use crate::{bounce::initial_bounce, follower::Follower};

#[derive(Component)]
pub struct Violin;

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_violin(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    following: Entity,
    follow_distance: f32,
    spawn_pos: Vec2,
) {
    let sprite_scale = 0.3;
    let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
    sprite_transform.scale = Vec3::new(sprite_scale, sprite_scale, 0.);

    commands.spawn((
        Violin,
        Follower {
            following,
            follow_distance,
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.),
        RigidBody::KinematicVelocityBased,
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule_y(30. * sprite_scale, 10. * sprite_scale),
        Visibility::default(),
        children![(
            AseAnimation {
                animation: Animation::tag("idle-right")
                    .with_repeat(AnimationRepeat::Loop)
                    .with_direction(AnimationDirection::Forward)
                    .with_speed(0.5),
                aseprite: asset_server.load("sprites/violin.aseprite"),
            },
            Sprite::default(),
            sprite_transform,
            initial_bounce(1.2)
        )],
    ));
}

#[derive(Component)]
pub struct Tuba;

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_tuba(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    following: Entity,
    follow_distance: f32,
    spawn_pos: Vec2,
) {
    let sprite_scale = 0.4;
    let mut sprite_transform = Transform::from_xyz(0., 0., 1.);
    sprite_transform.scale = Vec3::new(sprite_scale, sprite_scale, 0.);

    commands.spawn((
        Tuba,
        Follower {
            following,
            follow_distance,
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.),
        RigidBody::KinematicVelocityBased,
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule_y(30. * sprite_scale, 10. * sprite_scale),
        Visibility::default(),
        children![(
            AseAnimation {
                animation: Animation::tag("idle-right")
                    .with_repeat(AnimationRepeat::Loop)
                    .with_direction(AnimationDirection::Forward)
                    .with_speed(0.5),
                aseprite: asset_server.load("sprites/tuba.aseprite"),
            },
            Sprite::default(),
            sprite_transform,
            initial_bounce(1.2)
        )],
    ));
}
