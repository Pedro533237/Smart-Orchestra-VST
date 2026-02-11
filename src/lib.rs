use nih_plug::prelude::*;
use std::sync::Arc;

pub mod engine;

use engine::{Articulation, MAX_VOICES, MidiProcessor, SmoothedValue, Voice};

pub struct SmartOrchestraVST {
    params: Arc<SmartParams>,
    voices: Vec<Voice>,
    midi: MidiProcessor,
    cc1: SmoothedValue,
    cc11: SmoothedValue,
    sample_rate: f32,
    global_sample: i64,
}

#[derive(Params)]
struct SmartParams {
    #[id = "output"]
    pub output_gain: FloatParam,

    #[id = "cutoff"]
    pub cutoff_hz: FloatParam,
}

impl Default for SmartOrchestraVST {
    fn default() -> Self {
        Self {
            params: Arc::new(SmartParams::default()),
            voices: (0..MAX_VOICES).map(|_| Voice::new()).collect(),
            midi: MidiProcessor::new(),
            cc1: SmoothedValue::new(0.5),
            cc11: SmoothedValue::new(1.0),
            sample_rate: 44100.0,
            global_sample: 0,
        }
    }
}

impl Default for SmartParams {
    fn default() -> Self {
        Self {
            output_gain: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 6.0,
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            cutoff_hz: FloatParam::new(
                "LP Cutoff",
                10000.0,
                FloatRange::Skewed {
                    min: 150.0,
                    max: 18000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
        }
    }
}

impl Plugin for SmartOrchestraVST {
    const NAME: &'static str = "SmartOrchestraVST";
    const VENDOR: &'static str = "Pedro Audio Labs";
    const URL: &'static str = "https://pedroaudiolabs.local";
    const EMAIL: &'static str = "support@pedroaudiolabs.local";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: Some(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn reset(&mut self) {
        self.global_sample = 0;
        for voice in &mut self.voices {
            *voice = Voice::new();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();

        for (sample_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() as usize > sample_idx {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => self.handle_note_on(note, velocity),
                    NoteEvent::NoteOff { note, .. } => self.handle_note_off(note),
                    NoteEvent::MidiCC { cc, value, .. } => self.handle_cc(cc, value),
                    _ => {}
                }
                next_event = context.next_event();
            }

            let cc1 = self.cc1.next();
            let cc11 = self.cc11.next();
            let dyn_mod = 0.4 + cc1 * 0.75;
            let expression = cc11;
            let cutoff_hz = self.params.cutoff_hz.smoothed.next();
            let output_amp = util::db_to_gain(self.params.output_gain.smoothed.next());

            let mut left = 0.0;
            let mut right = 0.0;

            for voice in &mut self.voices {
                if voice.active {
                    voice.set_layer_gain(dyn_mod, self.sample_rate);
                    let (l, r) = voice.render(self.sample_rate, cutoff_hz);
                    left += l;
                    right += r;
                }
            }

            if let Some(s) = channel_samples.get_mut(0) {
                *s = left * expression * output_amp;
            }
            if let Some(s) = channel_samples.get_mut(1) {
                *s = right * expression * output_amp;
            }
            self.global_sample += 1;
        }

        ProcessStatus::Normal
    }
}

impl SmartOrchestraVST {
    fn handle_note_on(&mut self, note: u8, velocity_norm: f32) {
        let velocity = (velocity_norm.clamp(0.0, 1.0) * 127.0) as u8;
        let (_, layer_gain) = self.midi.detect_layer(velocity);
        let legato = self.midi.legato_engine.note_on(note as i32, self.global_sample);

        let articulation = if legato {
            Articulation::Sustain
        } else {
            self.midi.detect_articulation(500.0)
        };

        self.midi.step_round_robin();
        let rr_detune = (self.midi.round_robin as f32 - 1.5) * 0.03;
        let humanization = self.midi.humanize() + rr_detune;

        if let Some(voice) = self.voices.iter_mut().find(|v| !v.active) {
            voice.start(
                note,
                velocity,
                articulation,
                layer_gain,
                legato,
                self.sample_rate,
                self.global_sample,
                humanization,
            );
        }
    }

    fn handle_note_off(&mut self, note: u8) {
        self.midi.legato_engine.note_off(self.global_sample);
        for voice in &mut self.voices {
            if voice.active && voice.note == note {
                let duration_ms = ((self.global_sample - voice.start_sample) as f32 / self.sample_rate) * 1000.0;
                voice.articulation = self.midi.detect_articulation(duration_ms);
                voice.note_off(self.sample_rate);
            }
        }
    }

    fn handle_cc(&mut self, cc: u8, value: f32) {
        match cc {
            1 => self.cc1.set_target(value.clamp(0.0, 1.0), 5.0, self.sample_rate),
            11 => self.cc11.set_target(value.clamp(0.0, 1.0), 5.0, self.sample_rate),
            _ => {}
        }
    }
}

impl ClapPlugin for SmartOrchestraVST {
    const CLAP_ID: &'static str = "com.pedroaudiolabs.smartorchestravst";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Smart orchestral performance plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

impl Vst3Plugin for SmartOrchestraVST {
    const VST3_CLASS_ID: [u8; 16] = *b"SmrtOrchstrVST3!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(SmartOrchestraVST);
nih_export_vst3!(SmartOrchestraVST);
