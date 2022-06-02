use std::collections::VecDeque;
use anyhow::Result;

use apres::MIDI as MidiFile;
use apres::MIDIEvent as MidiEvent;
use apres::MIDIBytes as MidiBytes;

use super::format::Event;

impl Event {
    pub fn from_midi(ev: MidiEvent) -> Option<Self> {
        match ev {
            MidiEvent::NoteOn(_ch, note, _vel) => {
                match note {
                    59 => Some(Event::Strobe),
                    0..59 => Some(Event::Toggle { id: note }),

                    60 => Some(Event::Kick),
                    61 => Some(Event::Snare),
                    62 => Some(Event::Hat),
                    63.. => Some(Event::Trigger { id: note }),
                }
            },
            _ => {
                let bytes = ev.as_bytes();
                match bytes[0] >> 4 {
                    0xC => Some(Event::Program { id: bytes[1] }),
                    0xE => {
                        let lsb = bytes[1] as u16;
                        let msb = bytes[2] as u16;
                        Some(Event::Mod {
                            id: bytes[0] & 0xF,
                            fr: ((msb << 7) | lsb) as f32 / 16383.0,
                        })
                    },
                    _ => None,
                }
            }
        }
    }
}

pub fn parse_events(file: &str) -> Result<Vec<(f32, Event)>> {
    let file = MidiFile::from_path(file).unwrap();

    let mut midi: Vec<(usize, MidiEvent)> = file.get_tracks()
        .pop().unwrap().into_iter()
        .map(|(tick, id)| (tick, file.get_event(id).unwrap()))
        .collect();

    let ticks_per_quarter = file.get_ppqn();
    let mut us_per_quarter = 500_000;
    let mut last = 0.0;

    let tick_t = |ticks: usize, us_per_quarter: u32| {
        let us_per_tick = us_per_quarter as f32 / ticks_per_quarter as f32;
        let s_per_tick = us_per_tick / 1_000_000.0;
        let t = ticks as f32 * s_per_tick;
        t
    };

    let mut events = Vec::new();
    for (tick, midi) in midi.drain(..) {
        let t = last + tick_t(tick, us_per_quarter);
        last = t;

        match midi {
            MidiEvent::SetTempo(tempo) => us_per_quarter = tempo,
            _ => if let Some(event) = Event::from_midi(midi) {
                events.push((t, event));
            }
        }
    }

    Ok(events)
}

// impl Midi {
//     pub fn update(&mut self, t: f32) {
//         while self.events.len() != 0 {
//             let (next_tick, ev) = self.events.last().unwrap().clone();
//             let t_trigger = self.last + self.ticks_to_t(next_tick);

//             if t >= t_trigger {
//                 self.events.pop().unwrap();
//                 self.last = t_trigger;

//                 match ev {
//                     MidiEvent::SetTempo(tempo) => self.us_per_quarter = tempo,
//                     _ => if let Some(ev) = Event::from_midi(ev) {
//                         self.queue.push_back(ev);
//                     }
//                 }
//             } else {
//                 break;
//             }
//         }
//     }

// }
