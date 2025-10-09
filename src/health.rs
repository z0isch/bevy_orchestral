use bevy::{
    asset::Assets,
    camera::visibility::Visibility,
    color::Color,
    ecs::prelude::*,
    math::primitives::Rectangle,
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};

use crate::enemy::Enemy;

#[derive(Component, Debug)]
pub struct Health {
    pub max_health: u128,
    pub current_health: u128,
}

pub fn despawn_enemy_on_zero_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), (With<Enemy>, Changed<Health>)>,
) {
    for (entity, health) in query {
        if health.current_health == 0 {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Bundle)]
pub struct HealthBarBundle {
    health_bar: HealthBar,
    transform: Transform,
    visibility: Visibility,
}

pub const fn health_bar_bundle() -> HealthBarBundle {
    HealthBarBundle {
        health_bar: HealthBar,
        transform: Transform::from_xyz(0., 10., 1.),
        visibility: Visibility::Hidden,
    }
}

#[derive(Component)]
pub struct CurrentHealthBar;

#[allow(clippy::needless_pass_by_value)]
pub fn on_health_bar_add(
    event: On<Add, HealthBar>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut health_bar = commands.entity(event.entity);
    health_bar.with_children(|p| {
        p.spawn((
            Mesh2d(meshes.add(Rectangle::new(18., 2.))),
            MeshMaterial2d(materials.add(Color::hsva(0., 0., 1., 1.))),
            Transform::from_xyz(0., 0., 0.),
        ));
        p.spawn((
            CurrentHealthBar,
            Mesh2d(meshes.add(Rectangle::new(18., 2.))),
            MeshMaterial2d(materials.add(Color::hsva(1., 1., 1., 1.))),
            Transform::from_xyz(0., 0., 1.),
        ));
    });
}

pub fn health_bar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    health_query: Query<(&Health, &Children), Changed<Health>>,
    mut health_bar_query: Query<(&mut Visibility, &Children), With<HealthBar>>,
    current_health_bar_query: Query<Entity, With<CurrentHealthBar>>,
) {
    for (health, children) in health_query.iter() {
        for child in children.iter() {
            if let Ok((mut health_bar_visibility, health_bar_children)) =
                health_bar_query.get_mut(child)
            {
                if health.current_health != health.max_health {
                    *health_bar_visibility = Visibility::Visible;
                }

                for heatlh_bar_child in health_bar_children.iter() {
                    if let Ok(current_health_bar) = current_health_bar_query.get(heatlh_bar_child) {
                        #[allow(clippy::cast_precision_loss)]
                        let health_missing =
                            (health.current_health as f32 / health.max_health as f32) * 18.;
                        commands.entity(current_health_bar).try_insert((
                            Mesh2d(meshes.add(Rectangle::new(health_missing, 2.))),
                            Transform::from_xyz(-(9. - health_missing / 2.), 0., 1.),
                        ));
                    }
                }
            }
        }
    }
}
