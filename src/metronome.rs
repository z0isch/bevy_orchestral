use bevy::prelude::*;
use fraction::Fraction;

#[derive(Resource)]
pub struct Metronome {
    pub beat: u8,
    pub bpm: u64,
    pub is_beat_start_frame: bool,
    pub nanos_accumulated: Fraction,
    pub started: bool,
}

pub fn initial_metronome(bpm: u64) -> Metronome {
    return Metronome {
        beat: 0,
        bpm: bpm,
        is_beat_start_frame: false,
        nanos_accumulated: Fraction::from(0),
        started: false,
    };
}

fn nanos_per_beat(bpm: u64) -> Fraction {
    return Fraction::new(60_000_000_000u64, bpm * 4);
}

pub fn within_nanos_window(metronome: &Metronome, beat: u8, nanos_window: Fraction) -> bool {
    let shifted = metronome.beat as i8 - beat as i8;
    let on_beat_or_after_beat = shifted >= 0 && shifted <= 8;
    let since_start_or_till_next = if on_beat_or_after_beat {
        metronome.nanos_accumulated
    } else {
        nanos_per_beat(metronome.bpm) - metronome.nanos_accumulated
    };
    let beat_distance = 8 - (shifted.abs() - 8).abs();
    let factor = (beat_distance - 1).max(0);
    let base = since_start_or_till_next + nanos_per_beat(metronome.bpm) * factor;
    base <= nanos_window
}

pub fn metronome_system(time: Res<Time>, mut metronome: ResMut<Metronome>) {
    if metronome.started {
        let nanos_per_beat = nanos_per_beat(metronome.bpm);
        metronome.nanos_accumulated += Fraction::from(time.delta().as_nanos());
        if metronome.nanos_accumulated >= nanos_per_beat {
            metronome.is_beat_start_frame = true;
            metronome.beat = (metronome.beat + 1) % 16;
            metronome.nanos_accumulated -= nanos_per_beat;
        } else {
            metronome.is_beat_start_frame = false;
        }
    }
}
