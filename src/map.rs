use bevy::prelude::*;
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};
use bevy_rapier2d::prelude::{Collider, RigidBody};
use rand::{Rng, rng};

use crate::{bounce::initial_tile_bounce, window_size::WindowSize};

#[derive(Component, Debug)]
pub struct BlocksProjectiles;

#[allow(clippy::needless_pass_by_value)]
pub fn setup_map(
    window_size: Res<WindowSize>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let texture_handle: Handle<Image> = asset_server.load("sprites/kenney_tiny-town/tilemap.png");
    let map_size = TilemapSize {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_sign_loss)]
        x: (window_size.width as f32 / 2.0 / tile_size.x) as u32,
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_sign_loss)]
        y: (window_size.height as f32 / 2.0 / tile_size.y) as u32,
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let mut rng = rng();
            let texture_index = if rng.random_range(0..100) < 95 {
                0
            } else if rng.random_range(0..100) < 90 {
                1
            } else {
                2
            };

            let tile = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(texture_index),
                ..Default::default()
            });
            tile_storage.set(&tile_pos, tile.id());
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });

    // Layer 2
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let mut rng = rng();
            let tile_pos = TilePos { x, y };
            let texture_index = if x == 0 && y == 0 {
                Some((68, None)) // bottom-left corner
            } else if x == map_size.x - 1 && y == 0 {
                Some((70, None)) // bottom-right corner
            } else if x == 0 && y == map_size.y - 1 {
                Some((44, None)) // top-left corner
            } else if x == map_size.x - 1 && y == map_size.y - 1 {
                Some((46, None)) // top-right corner
            } else if y == 0 || y == map_size.y - 1 {
                Some((45, None)) // bottom edge or top edge
            } else if x == 0 || x == map_size.x - 1 {
                Some((58, None)) // left or right edge
            } else if rng.random_range(0..100) > 98 {
                Some((29, Some(132)))
            } else {
                None
            };

            if let Some((texture_index, tile_bounce)) = texture_index {
                let tile_pos_in_world = tile_pos.center_in_world(
                    &map_size,
                    &grid_size,
                    &tile_size,
                    &map_type,
                    &TilemapAnchor::Center,
                );
                let mut tile = commands.spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(texture_index),
                        ..Default::default()
                    },
                    Transform::from_xyz(tile_pos_in_world.x, tile_pos_in_world.y, 1.),
                ));
                let on_edge = x == 0 || y == 0 || x == map_size.x - 1 || y == map_size.y - 1;
                if !on_edge {
                    tile.insert((
                        Collider::ball(tile_size.x / 2.),
                        RigidBody::Fixed,
                        BlocksProjectiles,
                    ));
                }
                if let Some(tile_bounce) = tile_bounce {
                    tile.insert(initial_tile_bounce(TileTextureIndex(tile_bounce)));
                }
                tile_storage.set(&tile_pos, tile.id());
            }
        }
    }
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0., 0., 2.0),
        ..Default::default()
    });
    #[allow(clippy::cast_precision_loss)]
    let height_offset = window_size.height as f32 / 4.;
    #[allow(clippy::cast_precision_loss)]
    let width_offset = window_size.width as f32 / 4.;

    commands.spawn((
        BlocksProjectiles,
        RigidBody::Fixed,
        Transform::from_xyz(0., 1000. - tile_size.y + height_offset, 0.),
        Collider::cuboid(width_offset, 1000.),
    ));
    commands.spawn((
        BlocksProjectiles,
        RigidBody::Fixed,
        Transform::from_xyz(0., -1000. + tile_size.y - height_offset, 0.),
        Collider::cuboid(width_offset, 1000.),
    ));
    commands.spawn((
        BlocksProjectiles,
        RigidBody::Fixed,
        Transform::from_xyz(1000. - tile_size.x + width_offset, 0., 0.),
        Collider::cuboid(1000., height_offset),
    ));
    commands.spawn((
        BlocksProjectiles,
        RigidBody::Fixed,
        Transform::from_xyz(-1000. + tile_size.x - width_offset, 0., 0.),
        Collider::cuboid(1000., height_offset),
    ));
}
