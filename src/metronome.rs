use bevy::{prelude::*, time::Stopwatch};
use fraction::Fraction;

#[derive(Resource, Clone)]
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

fn nanos_fraction_per_beat(bpm: u64) -> Fraction {
    return Fraction::new(60_000_000_000u64, bpm * 4);
}

pub fn nanos_per_beat(bpm: u64) -> u64 {
    nanos_fraction_per_beat(bpm).floor().try_into().unwrap()
}

pub fn closest_beat(metronome: &Metronome) -> u8 {
    if metronome.nanos_accumulated < nanos_fraction_per_beat(metronome.bpm) / 2 {
        metronome.beat
    } else if metronome.beat == 0 {
        15
    } else {
        metronome.beat - 1
    }
}

pub fn within_nanos_window(metronome: &Metronome, beat: u8, nanos_window: Fraction) -> bool {
    let shifted = metronome.beat as i8 - beat as i8;
    let on_beat_or_after_beat = shifted >= 0 && shifted <= 8;
    let since_start_or_till_next = if on_beat_or_after_beat {
        metronome.nanos_accumulated
    } else {
        nanos_fraction_per_beat(metronome.bpm) - metronome.nanos_accumulated
    };
    let beat_distance = 8 - (shifted.abs() - 8).abs();
    let factor = (beat_distance - 1).max(0);
    let base = since_start_or_till_next + nanos_fraction_per_beat(metronome.bpm) * factor;
    base <= nanos_window
}

pub fn metronome_system(time: Res<Time>, mut metronome: ResMut<Metronome>) {
    if metronome.started {
        let nanos_per_beat = nanos_fraction_per_beat(metronome.bpm);
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

pub fn is_down_beat(metronome: &Metronome) -> bool {
    down_beats(metronome).contains(&metronome.beat)
}

pub fn down_beats(_metronome: &Metronome) -> Vec<u8> {
    vec![0, 4, 8, 12]
}

pub struct MetronomeTimer {
    pub number_beats_duration: u8,
    timer_state: MetronomeTimerState,
    pub stopwatch: Stopwatch,
}

enum MetronomeTimerState {
    NotStarted,
    Running { beats_elapsed: i8 },
}

impl MetronomeTimer {
    pub fn new(number_beats_duration: u8) -> MetronomeTimer {
        MetronomeTimer {
            number_beats_duration,
            timer_state: MetronomeTimerState::NotStarted,
            stopwatch: Stopwatch::new(),
        }
    }
    pub fn tick(&mut self, metronome: &Metronome, time: Time) {
        if !metronome.started {
            return;
        }
        self.stopwatch.tick(time.delta());

        match self.timer_state {
            MetronomeTimerState::NotStarted => {
                self.timer_state = MetronomeTimerState::Running {
                    beats_elapsed: if closest_beat(&metronome) == metronome.beat {
                        0
                    } else {
                        -1
                    },
                };
            }
            MetronomeTimerState::Running {
                ref mut beats_elapsed,
            } => {
                if (*beats_elapsed).try_into().unwrap_or(0 as u8) == self.number_beats_duration {
                    self.stopwatch.reset();
                    *beats_elapsed = 0;
                }
                if metronome.is_beat_start_frame {
                    *beats_elapsed += 1;
                }
            }
        }
    }
    pub fn just_finished(&self, metronome: &Metronome) -> bool {
        match self.timer_state {
            MetronomeTimerState::NotStarted => false,
            MetronomeTimerState::Running { beats_elapsed, .. } => {
                let beats_elapsed: u8 = beats_elapsed.try_into().unwrap_or(0);
                metronome.is_beat_start_frame && beats_elapsed == self.number_beats_duration
            }
        }
    }
}
