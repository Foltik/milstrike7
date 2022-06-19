use anyhow::Result;
use std::collections::VecDeque;

use apres::MIDIBytes as MidiBytes;
use apres::MIDIEvent as MidiEvent;
use apres::MIDI as MidiFile;

use super::format::Event;

impl Event {
    // pub fn from_midi(ev: MidiEvent) -> Option<Self> {
    //     match ev {
    //         MidiEvent::NoteOn(_ch, note, _vel) => {
    //             match note {
    //                 0..60 => Some(Event::Trigger { id: note }),

    //                 60 => Some(Event::Kick),
    //                 61 => Some(Event::Snare),
    //                 62 => Some(Event::Hat),
    //                 63.. => Some(Event::Trigger { id: note }),
    //             }
    //         },
    //         _ => {
    //             let bytes = ev.as_bytes();
    //             match bytes[0] >> 4 {
    //                 0xC => Some(Event::Program { id: bytes[1] }),
    //                 0xE => {
    //                     let lsb = bytes[1] as u16;
    //                     let msb = bytes[2] as u16;
    //                     Some(Event::Mod {
    //                         id: bytes[0] & 0xF,
    //                         fr: ((msb << 7) | lsb) as f32 / 16383.0,
    //                     })
    //                 },
    //                 _ => None,
    //             }
    //         }
    //     }
    // }
}

pub fn parse_events(file: &str) -> Result<Vec<(f32, Event)>> {
    let file = MidiFile::from_path(file).unwrap();

    let mut midi: Vec<(usize, MidiEvent)> = file
        .get_tracks()
        .pop()
        .unwrap()
        .into_iter()
        .map(|(tick, id)| (tick, file.get_event(id).unwrap()))
        .collect();

    let ticks_per_quarter = file.get_ppqn();
    // let mut us_per_quarter = 500_000;
    // 141.371 BPM
    // x0.5 = 70.6855
    let mut us_per_quarter = 500_000.0;
    let mut last = 0.0;

    let tick_t = |ticks: usize, us_per_quarter: f32| {
        let us_per_tick = us_per_quarter as f32 / ticks_per_quarter as f32;
        let s_per_tick = us_per_tick / 1_000_000.0;
        let t = ticks as f32 * s_per_tick;
        t
    };

    let mut midis = Vec::new();
    for (tick, midi) in midi.drain(..) {
        let t = last + tick_t(tick, us_per_quarter);
        last = t;

        match midi {
            MidiEvent::SetTempo(tempo) => {
                println!("Tempo: {}", 60.0 / (tempo as f32 / 1_000_000.0));
                us_per_quarter = tempo as f32;
            }
            _ => midis.push((t, midi)),
        }
    }

    let mut events = Vec::new();
    for (i, (t0, midi)) in midis.iter().enumerate() {
        if let Some(event) = match midi {
            MidiEvent::NoteOn(_ch, id, _vel) => match id {
                0..30 => Some(Event::Trigger { id: *id }),
                30..60 => Some(Event::Toggle { id: *id, state: true }),
                60.. => {
                    let t1 = midis[i..]
                        .iter()
                        .find(|(t, midi)| match midi {
                            MidiEvent::NoteOff(_, _id, _) => *_id == *id,
                            _ => false,
                        })
                        .map(|(t, midi)| t)
                        .unwrap_or(t0);

                    Some(Event::Beat {
                        id: *id,
                        t: t1 - t0,
                    })
                }
            },
            MidiEvent::NoteOff(_ch, id, _vel) => match id {
                30..60 => Some(Event::Toggle { id: *id, state: false }),
                _ => None,
            },
            _ => {
                let bytes = midi.as_bytes();
                match bytes[0] >> 4 {
                    0xE => {
                        let lsb = bytes[1] as u16;
                        let msb = bytes[2] as u16;
                        Some(Event::Mod {
                            id: bytes[0] & 0xF,
                            fr: ((msb << 7) | lsb) as f32 / 16383.0,
                        })
                    }
                    _ => None,
                }
            }
        } {
            events.push((*t0, event));
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
