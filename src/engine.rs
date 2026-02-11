use rand::{rngs::SmallRng, Rng, SeedableRng};

pub const MAX_VOICES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Articulation {
    Staccato,
    Marcato,
    Sustain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicLayer {
    Pp,
    P,
    Mp,
    Mf,
    F,
    Ff,
}

#[derive(Debug, Clone, Copy)]
pub struct SmoothedValue {
    current: f32,
    target: f32,
    step: f32,
    samples_left: usize,
}

impl SmoothedValue {
    pub fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
            samples_left: 0,
        }
    }

    pub fn set_target(&mut self, target: f32, time_ms: f32, sample_rate: f32) {
        self.target = target;
        let samples = ((time_ms / 1000.0) * sample_rate).max(1.0) as usize;
        self.samples_left = samples;
        self.step = (self.target - self.current) / samples as f32;
    }

    pub fn set_immediate(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.samples_left = 0;
    }

    pub fn next(&mut self) -> f32 {
        if self.samples_left > 0 {
            self.current += self.step;
            self.samples_left -= 1;
            if self.samples_left == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    stage: EnvelopeStage,
    value: f32,
    attack_samps: f32,
    decay_samps: f32,
    sustain_level: f32,
    release_samps: f32,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            value: 0.0,
            attack_samps: 1.0,
            decay_samps: 1.0,
            sustain_level: 0.8,
            release_samps: 1.0,
        }
    }

    pub fn trigger(&mut self, articulation: Articulation, legato: bool, sample_rate: f32) {
        let (attack_ms, decay_ms, sustain) = if legato {
            (30.0, 160.0, 0.85)
        } else {
            match articulation {
                Articulation::Staccato => (2.0, 45.0, 0.35),
                Articulation::Marcato => (10.0, 90.0, 0.6),
                Articulation::Sustain => (20.0, 160.0, 0.82),
            }
        };

        self.attack_samps = ms_to_samples(attack_ms, sample_rate);
        self.decay_samps = ms_to_samples(decay_ms, sample_rate);
        self.release_samps = ms_to_samples(90.0, sample_rate);
        self.sustain_level = sustain;
        self.stage = EnvelopeStage::Attack;
    }

    pub fn release(&mut self, release_ms: f32, sample_rate: f32) {
        self.release_samps = ms_to_samples(release_ms, sample_rate);
        self.stage = EnvelopeStage::Release;
    }

    pub fn next(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => 0.0,
            EnvelopeStage::Attack => {
                self.value += 1.0 / self.attack_samps;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
                self.value
            }
            EnvelopeStage::Decay => {
                let step = (1.0 - self.sustain_level) / self.decay_samps;
                self.value -= step;
                if self.value <= self.sustain_level {
                    self.value = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
                self.value
            }
            EnvelopeStage::Sustain => self.value,
            EnvelopeStage::Release => {
                self.value -= (self.value.max(0.0001)) / self.release_samps;
                if self.value <= 0.0005 {
                    self.value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
                self.value
            }
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == EnvelopeStage::Idle
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LegatoEngine {
    active_note_end_sample: i64,
    last_note_off_sample: i64,
    last_note: i32,
    pub is_legato: bool,
}

impl LegatoEngine {
    pub fn new() -> Self {
        Self {
            active_note_end_sample: -1,
            last_note_off_sample: -1000,
            last_note: -1,
            is_legato: false,
        }
    }

    pub fn note_on(&mut self, note: i32, global_sample: i64) -> bool {
        let overlap = self.active_note_end_sample >= global_sample;
        let gap_legato = global_sample - self.last_note_off_sample < 30;
        self.is_legato = overlap || gap_legato;
        self.last_note = note;
        self.is_legato
    }

    pub fn note_off(&mut self, global_sample: i64) {
        self.last_note_off_sample = global_sample;
        self.active_note_end_sample = global_sample;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Voice {
    pub active: bool,
    pub note: u8,
    velocity: u8,
    phase_saw: f32,
    phase_sine: f32,
    freq: f32,
    pub envelope: Envelope,
    pub articulation: Articulation,
    pub start_sample: i64,
    pub legato_amount: f32,
    dynamic_gain: SmoothedValue,
    pan: f32,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            active: false,
            note: 0,
            velocity: 0,
            phase_saw: 0.0,
            phase_sine: 0.0,
            freq: 440.0,
            envelope: Envelope::new(),
            articulation: Articulation::Sustain,
            start_sample: 0,
            legato_amount: 0.0,
            dynamic_gain: SmoothedValue::new(0.5),
            pan: 0.5,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        note: u8,
        velocity: u8,
        articulation: Articulation,
        layer_gain: f32,
        legato: bool,
        sample_rate: f32,
        global_sample: i64,
        humanization: f32,
    ) {
        self.active = true;
        self.note = note;
        self.velocity = velocity;
        self.articulation = articulation;
        self.freq = midi_note_to_hz(note as f32 + humanization);
        self.start_sample = global_sample;
        self.legato_amount = if legato { 0.08 } else { 0.0 };
        self.dynamic_gain.set_immediate(layer_gain);
        self.envelope.trigger(articulation, legato, sample_rate);
        self.pan = 0.5 + humanization * 0.03;
    }

    pub fn note_off(&mut self, sample_rate: f32) {
        let release = match self.articulation {
            Articulation::Staccato => 45.0,
            Articulation::Marcato => 100.0,
            Articulation::Sustain => 180.0,
        };
        self.envelope.release(release, sample_rate);
    }

    pub fn render(&mut self, sample_rate: f32, cutoff_hz: f32) -> (f32, f32) {
        if !self.active {
            return (0.0, 0.0);
        }

        let glide_target = midi_note_to_hz(self.note as f32);
        self.freq += (glide_target - self.freq) * self.legato_amount;

        let inc = self.freq / sample_rate;
        self.phase_saw = (self.phase_saw + inc) % 1.0;
        self.phase_sine = (self.phase_sine + inc) % 1.0;

        let saw = self.phase_saw * 2.0 - 1.0;
        let sine = (self.phase_sine * std::f32::consts::TAU).sin();
        let mut sample = saw * 0.65 + sine * 0.35;

        let cutoff_norm = (cutoff_hz / (sample_rate * 0.5)).clamp(0.001, 0.99);
        sample *= cutoff_norm;

        sample *= self.envelope.next() * self.dynamic_gain.next();

        if self.envelope.is_idle() {
            self.active = false;
            return (0.0, 0.0);
        }

        let left = sample * (1.0 - self.pan).sqrt();
        let right = sample * self.pan.sqrt();
        (left, right)
    }

    pub fn set_layer_gain(&mut self, target: f32, sample_rate: f32) {
        self.dynamic_gain.set_target(target, 5.0, sample_rate);
    }
}

#[derive(Debug)]
pub struct MidiProcessor {
    pub legato_engine: LegatoEngine,
    rng: SmallRng,
    pub round_robin: usize,
}

impl MidiProcessor {
    pub fn new() -> Self {
        Self {
            legato_engine: LegatoEngine::new(),
            rng: SmallRng::seed_from_u64(0xA11CE55),
            round_robin: 0,
        }
    }

    pub fn detect_layer(&self, velocity: u8) -> (DynamicLayer, f32) {
        match velocity {
            0..=29 => (DynamicLayer::Pp, 0.20),
            30..=49 => (DynamicLayer::P, 0.32),
            50..=69 => (DynamicLayer::Mp, 0.45),
            70..=89 => (DynamicLayer::Mf, 0.6),
            90..=109 => (DynamicLayer::F, 0.78),
            _ => (DynamicLayer::Ff, 0.95),
        }
    }

    pub fn detect_articulation(&self, duration_ms: f32) -> Articulation {
        if duration_ms < 120.0 {
            Articulation::Staccato
        } else if duration_ms <= 400.0 {
            Articulation::Marcato
        } else {
            Articulation::Sustain
        }
    }

    pub fn humanize(&mut self) -> f32 {
        self.rng.gen_range(-0.12..0.12)
    }

    pub fn step_round_robin(&mut self) {
        self.round_robin = (self.round_robin + 1) % 4;
    }
}

#[inline]
pub fn midi_note_to_hz(note: f32) -> f32 {
    440.0 * (2.0_f32).powf((note - 69.0) / 12.0)
}

#[inline]
fn ms_to_samples(ms: f32, sample_rate: f32) -> f32 {
    ((ms / 1000.0) * sample_rate).max(1.0)
}
