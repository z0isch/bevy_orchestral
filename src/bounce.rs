use bevy::prelude::*;

use crate::metronome::Metronome;

#[derive(Component)]
pub struct Bounce {
    pub scale: f32,
}

pub fn bounce_system(metronome: Res<Metronome>, mut bouncers: Query<(&mut Transform, &Bounce)>) {
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
