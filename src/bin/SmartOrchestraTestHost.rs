use anyhow::{Context, Result};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use smart_orchestra_vst::engine::{MidiProcessor, SmoothedValue, Voice, MAX_VOICES};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Copy)]
struct ScheduledEvent {
    sample: usize,
    kind: EventKind,
}

#[derive(Debug, Clone, Copy)]
enum EventKind {
    NoteOn { note: u8, vel: u8 },
    NoteOff { note: u8 },
    Cc { cc: u8, value: f32 },
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Uso: {} <arquivo.mid> <saida.wav> [sample_rate]\nExemplo: {} demo.mid out.wav 48000",
            args[0], args[0]
        );
        std::process::exit(1);
    }

    let midi_path = PathBuf::from(&args[1]);
    let wav_path = PathBuf::from(&args[2]);
    let sample_rate = args.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(48_000);

    let midi_data = fs::read(&midi_path).with_context(|| format!("Falha ao ler MIDI: {midi_path:?}"))?;
    let smf = Smf::parse(&midi_data).context("Falha no parse do arquivo MIDI")?;

    let mut events = collect_events(&smf, sample_rate as f32)?;
    events.sort_by_key(|e| e.sample);

    let total_samples = events.last().map(|e| e.sample + sample_rate as usize * 2).unwrap_or(sample_rate as usize * 2);

    render_to_wav(events, total_samples, sample_rate, &wav_path)
}

fn collect_events(smf: &Smf<'_>, sample_rate: f32) -> Result<Vec<ScheduledEvent>> {
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(m) => m.as_int() as f32,
        Timing::Timecode(_, _) => anyhow::bail!("Timing SMPTE não suportado"),
    };

    let mut tempo_us_per_beat = 500_000.0;
    let mut out = Vec::new();

    for track in &smf.tracks {
        let mut abs_ticks: u64 = 0;
        for event in track {
            abs_ticks += event.delta.as_int() as u64;

            if let TrackEventKind::Meta(MetaMessage::Tempo(t)) = event.kind {
                tempo_us_per_beat = t.as_int() as f32;
            }

            let seconds = (abs_ticks as f32 / ticks_per_beat) * (tempo_us_per_beat / 1_000_000.0);
            let sample = (seconds * sample_rate) as usize;

            if let TrackEventKind::Midi { message, .. } = event.kind {
                match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => out.push(ScheduledEvent {
                        sample,
                        kind: EventKind::NoteOn {
                            note: key.as_int(),
                            vel: vel.as_int(),
                        },
                    }),
                    MidiMessage::NoteOn { key, .. } | MidiMessage::NoteOff { key, .. } => {
                        out.push(ScheduledEvent {
                            sample,
                            kind: EventKind::NoteOff { note: key.as_int() },
                        })
                    }
                    MidiMessage::Controller { controller, value } => {
                        let cc = controller.as_int();
                        if cc == 1 || cc == 11 {
                            out.push(ScheduledEvent {
                                sample,
                                kind: EventKind::Cc {
                                    cc,
                                    value: value.as_int() as f32 / 127.0,
                                },
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(out)
}

fn render_to_wav(events: Vec<ScheduledEvent>, total_samples: usize, sample_rate: u32, path: &PathBuf) -> Result<()> {
    let mut voices: Vec<Voice> = (0..MAX_VOICES).map(|_| Voice::new()).collect();
    let mut midi = MidiProcessor::new();
    let mut cc1 = SmoothedValue::new(0.4);
    let mut cc11 = SmoothedValue::new(1.0);
    let mut event_cursor = 0;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;

    for sample_idx in 0..total_samples {
        while event_cursor < events.len() && events[event_cursor].sample <= sample_idx {
            match events[event_cursor].kind {
                EventKind::NoteOn { note, vel } => {
                    let (_layer, layer_gain) = midi.detect_layer(vel);
                    let legato = midi.legato_engine.note_on(note as i32, sample_idx as i64);
                    let humanize = midi.humanize();
                    midi.step_round_robin();
                    if let Some(v) = voices.iter_mut().find(|v| !v.active) {
                        v.start(
                            note,
                            vel,
                            midi.detect_articulation(500.0),
                            layer_gain,
                            legato,
                            sample_rate as f32,
                            sample_idx as i64,
                            humanize,
                        );
                    }
                }
                EventKind::NoteOff { note } => {
                    midi.legato_engine.note_off(sample_idx as i64);
                    for v in &mut voices {
                        if v.active && v.note == note {
                            v.note_off(sample_rate as f32);
                        }
                    }
                }
                EventKind::Cc { cc, value } => match cc {
                    1 => cc1.set_target(value, 5.0, sample_rate as f32),
                    11 => cc11.set_target(value, 5.0, sample_rate as f32),
                    _ => {}
                },
            }
            event_cursor += 1;
        }

        let mut l = 0.0;
        let mut r = 0.0;
        let dyn_gain = 0.35 + cc1.next() * 0.75;
        let expr = cc11.next();

        for v in &mut voices {
            if v.active {
                v.set_layer_gain(dyn_gain, sample_rate as f32);
                let (vl, vr) = v.render(sample_rate as f32, 12_000.0);
                l += vl;
                r += vr;
            }
        }

        let scale = 0.22 * expr;
        let li = (l * scale * i32::MAX as f32) as i32;
        let ri = (r * scale * i32::MAX as f32) as i32;
        writer.write_sample(li)?;
        writer.write_sample(ri)?;
    }

    writer.finalize()?;
    println!("Render concluído em: {}", path.display());
    Ok(())
}
