use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;

use crate::metronome::Metronome;

#[derive(Component)]
pub struct Bounce {
    pub scale: f32,
}

pub fn bounce_system(metronome: Res<Metronome>, mut bouncers: Query<(&mut Transform, &Bounce)>) {
    if metronome.started {
        for (mut transform, bounce) in bouncers.iter_mut() {
            if metronome.is_beat_start_frame {
                if metronome.beat == 0 {
                    transform.scale *= bounce.scale;
                }
                if metronome.beat == 1 {
                    transform.scale /= bounce.scale;
                }

                if metronome.beat == 8 {
                    transform.scale *= bounce.scale;
                }
                if metronome.beat == 9 {
                    transform.scale /= bounce.scale;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct TileBounce {
    texture_index: TileTextureIndex,
    initial_texture_index: Option<TileTextureIndex>,
}

pub fn initial_tile_bounce(texture_index: TileTextureIndex) -> TileBounce {
    TileBounce {
        texture_index,
        initial_texture_index: None,
    }
}

pub fn tile_bounce_system(
    metronome: Res<Metronome>,
    mut bouncers: Query<(&mut TileBounce, &mut TileTextureIndex)>,
) {
    if metronome.started {
        for (mut bounce, mut tile_texture_index) in bouncers.iter_mut() {
            let initial_texture_index = *bounce
                .initial_texture_index
                .get_or_insert(*tile_texture_index);

            if metronome.is_beat_start_frame {
                if metronome.beat == 0 {
                    *tile_texture_index = bounce.texture_index;
                }
                if metronome.beat == 1 {
                    *tile_texture_index = initial_texture_index;
                }

                if metronome.beat == 8 {
                    *tile_texture_index = bounce.texture_index;
                }
                if metronome.beat == 9 {
                    *tile_texture_index = initial_texture_index;
                }
            }
        }
    }
}
