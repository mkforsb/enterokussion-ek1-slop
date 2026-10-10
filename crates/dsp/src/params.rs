//! Parameter definitions shared by the engine, the wasm worklet and the UI.
//!
//! Every parameter is stored as an `f32`. Continuous parameters are normalized
//! to `0.0..=1.0` (think "knob position"); choice parameters hold the index of
//! the selected option. The [`map`] module converts knob positions into
//! physical units and is used by both the DSP and the UI read-outs, so what the
//! panel displays is exactly what the engine does.
//!
//! The three macro knobs (X, Y, Z) mean something different for every engine;
//! [`macro_label`] and [`macro_display`] give their per-engine names.

/// Per-voice parameters. The discriminant is the stable wire id.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Param {
    /// Synthesis engine (see [`ENGINE_NAMES`]).
    Engine = 0,
    /// Pitch of chamber 1 (continuous, not quantized to notes).
    Pitch,
    /// Fine tune.
    Fine,
    /// Amplitude decay time of chamber 1 (T60).
    Decay,
    /// Engine macro 1.
    MacroX,
    /// Engine macro 2.
    MacroY,
    /// Engine macro 3.
    MacroZ,
    /// Chamber 1 level.
    Level1,
    /// Chamber 2 tuning relative to chamber 1, free (±24 semitones).
    Tune2,
    /// Chamber 2 level.
    Level2,
    /// Chamber 2 envelope length (attack, hold and decay) as a multiple of
    /// chamber 1's.
    Length2,
    /// Chamber 2 onset delay after the trigger.
    Delay2,
    /// Chamber 3 tuning relative to chamber 1.
    Tune3,
    /// Chamber 3 level.
    Level3,
    /// Chamber 3 envelope length as a multiple of chamber 1's.
    Length3,
    /// Chamber 3 onset delay after the trigger.
    Delay3,
    /// Peristalsis: contraction rate.
    PeriRate,
    /// Peristalsis: how far the pressure drops between contractions.
    PeriSqueeze,
    /// Peristalsis: pitch change at the peak of a contraction, bipolar.
    PeriPitch,
    /// Peristalsis: randomness of each contraction's timing and strength.
    PeriIrreg,
    /// Peristalsis: phase lag from one chamber to the next (a travelling wave).
    PeriTravel,
    /// Amplitude envelope attack.
    Attack,
    /// Attack curve, bipolar: a slow swell … linear … a fast snap.
    Curve,
    /// Amplitude envelope hold (the sustain before the decay starts).
    Hold,
    /// Fizz layer level.
    FizzLevel,
    /// Fizz layer band centre.
    FizzTone,
    /// Fizz: smooth hiss … sparse crackle.
    FizzCrackle,
    /// Pitch sweep time.
    SweepSpeed,
    /// Pitch sweep depth, bipolar (centre = off, up = starts high).
    SweepDepth,
    /// Chaos modulator (Lorenz attractor) rate.
    ChaosRate,
    /// Chaos modulator depth.
    ChaosDepth,
    /// Chaos modulator destination.
    ChaosDest,
    /// Per-hit random offsets of pitch and macros.
    Jitter,
    /// Filter cutoff.
    Cutoff,
    /// Filter resonance (ladder feedback).
    Reso,
    /// Envelope → cutoff amount, bipolar.
    FilterEnv,
    /// Filter type: LP12 / LP24 / BP / HP.
    FilterType,
    /// Saturator drive.
    Drive,
    /// Saturator curve.
    SatType,
    /// Saturator dry/wet.
    SatMix,
    /// Saturator position: before the filter or after the EQ.
    SatPos,
    /// EQ low cut (12 dB/oct highpass), off at zero.
    LowCut,
    /// EQ low shelf frequency.
    LowFreq,
    /// EQ low shelf gain, bipolar.
    LowGain,
    /// EQ mid bell frequency.
    MidFreq,
    /// EQ mid bell gain, bipolar.
    MidGain,
    /// EQ mid bell Q.
    MidQ,
    /// EQ high shelf frequency.
    HighFreq,
    /// EQ high shelf gain, bipolar.
    HighGain,
    /// Chorus wet amount.
    ChorusMix,
    /// Chorus LFO rate.
    ChorusRate,
    /// Chorus modulation depth.
    ChorusDepth,
    /// Chorus crossover: everything below it stays dry and mono, off at zero.
    ChorusXover,
    /// Channel output level.
    Level,
    /// Trigger sensitivity.
    Sense,
    /// Stereo position.
    Pan,
    /// Reverb send level (post channel output level).
    ReverbMix,
    /// Reverb decay time (≈ RT60).
    ReverbDecay,
    /// Reverb damping / brightness.
    ReverbTone,
    /// Reverb pre-delay.
    ReverbPredelay,
}

pub const PARAM_COUNT: usize = 60;

pub const ALL_PARAMS: [Param; PARAM_COUNT] = [
    Param::Engine,
    Param::Pitch,
    Param::Fine,
    Param::Decay,
    Param::MacroX,
    Param::MacroY,
    Param::MacroZ,
    Param::Level1,
    Param::Tune2,
    Param::Level2,
    Param::Length2,
    Param::Delay2,
    Param::Tune3,
    Param::Level3,
    Param::Length3,
    Param::Delay3,
    Param::PeriRate,
    Param::PeriSqueeze,
    Param::PeriPitch,
    Param::PeriIrreg,
    Param::PeriTravel,
    Param::Attack,
    Param::Curve,
    Param::Hold,
    Param::FizzLevel,
    Param::FizzTone,
    Param::FizzCrackle,
    Param::SweepSpeed,
    Param::SweepDepth,
    Param::ChaosRate,
    Param::ChaosDepth,
    Param::ChaosDest,
    Param::Jitter,
    Param::Cutoff,
    Param::Reso,
    Param::FilterEnv,
    Param::FilterType,
    Param::Drive,
    Param::SatType,
    Param::SatMix,
    Param::SatPos,
    Param::LowCut,
    Param::LowFreq,
    Param::LowGain,
    Param::MidFreq,
    Param::MidGain,
    Param::MidQ,
    Param::HighFreq,
    Param::HighGain,
    Param::ChorusMix,
    Param::ChorusRate,
    Param::ChorusDepth,
    Param::ChorusXover,
    Param::Level,
    Param::Sense,
    Param::Pan,
    Param::ReverbMix,
    Param::ReverbDecay,
    Param::ReverbTone,
    Param::ReverbPredelay,
];

pub const ENGINE_GURGLE: u32 = 0;
pub const ENGINE_FLATUS: u32 = 1;
pub const ENGINE_GROWL: u32 = 2;
pub const ENGINE_SQUEAK: u32 = 3;
pub const ENGINE_TRACT: u32 = 4;
pub const ENGINE_SYMBIO: u32 = 5;
pub const ENGINE_FLORA: u32 = 6;
pub const ENGINE_RUMBLE: u32 = 7;
pub const ENGINE_COUNT: usize = 8;

pub const ENGINE_NAMES: &[&str] = &[
    "GURGLE", "FLATUS", "GROWL", "SQUEAK", "TRACT", "SYMBIO", "FLORA", "RUMBLE",
];

/// One-line description of each engine, for the panel.
pub const ENGINE_BLURBS: [&str; ENGINE_COUNT] = [
    "Gas bubbles detaching in digestive fluid: a cloud of chirping Minnaert resonances",
    "A pressure-driven sphincter valve (Van der Pol) that buzzes, sputters and sags into a cheek cavity",
    "A creaky pulse source with jitter and subharmonics through a morphing set of formants",
    "Stick-slip friction rubbing a wet, rubbery membrane: squeaks, creaks and groans",
    "A long digestive tube (Kelly–Lochbaum) with a peristaltic constriction travelling through it",
    "Two symbiotic organisms phase-modulating each other until they lock, beat or break into chaos",
    "A chattering swarm of gut flora: chirping, gliding critters that talk in bursts",
    "Borborygmus: drifting, beating low partials, roughened by gas and grit",
];

pub const CHAOS_DEST_NAMES: &[&str] = &["PITCH", "X", "Y", "Z", "CUTOFF"];
pub const CHAOS_PITCH: u32 = 0;
pub const CHAOS_X: u32 = 1;
pub const CHAOS_Y: u32 = 2;
pub const CHAOS_Z: u32 = 3;
pub const CHAOS_CUTOFF: u32 = 4;

pub const FILTER_TYPE_NAMES: &[&str] = &["LP12", "LP24", "BP", "HP"];
pub const FILTER_LP12: u32 = 0;
pub const FILTER_LP24: u32 = 1;
pub const FILTER_BP: u32 = 2;
pub const FILTER_HP: u32 = 3;

pub const SAT_TYPE_NAMES: &[&str] = &["TAPE", "TUBE", "HARD", "FOLD"];
pub const SAT_TAPE: u32 = 0;
pub const SAT_TUBE: u32 = 1;
pub const SAT_HARD: u32 = 2;
pub const SAT_FOLD: u32 = 3;

pub const SAT_POS_NAMES: &[&str] = &["PRE", "POST"];
pub const SAT_PRE: u32 = 0;
pub const SAT_POST: u32 = 1;

/// GROWL: the vowels VOWEL morphs between.
pub const VOWEL_NAMES: [&str; 5] = ["OO", "OH", "AH", "EH", "EE"];

/// SQUEAK: the mode sets MEMBR morphs between.
pub const MEMBRANE_NAMES: [&str; 3] = ["TUBE", "MEMBRANE", "GRISTLE"];

/// TRACT: buzz … gas excitation.
pub const GAS_NAMES: [&str; 2] = ["BUZZ", "GAS"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamKind {
    Continuous,
    Choice(&'static [&'static str]),
}

#[derive(Clone, Copy, Debug)]
pub struct ParamInfo {
    pub param: Param,
    /// Descriptive name.
    pub name: &'static str,
    /// Panel label.
    pub label: &'static str,
    pub default: f32,
    pub kind: ParamKind,
}

impl Param {
    pub fn id(self) -> u32 {
        self as u32
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_id(id: u32) -> Option<Param> {
        ALL_PARAMS.get(id as usize).copied()
    }

    pub fn info(self) -> ParamInfo {
        use ParamKind::*;
        let (name, label, default, kind) = match self {
            Param::Engine => (
                "Engine",
                "ENGINE",
                ENGINE_FLATUS as f32,
                Choice(ENGINE_NAMES),
            ),
            Param::Pitch => ("Pitch", "PITCH", 0.3, Continuous),
            Param::Fine => ("Fine tune", "FINE", 0.5, Continuous),
            Param::Decay => ("Decay", "DECAY", 0.45, Continuous),
            Param::MacroX => ("Macro X", "X", 0.4, Continuous),
            Param::MacroY => ("Macro Y", "Y", 0.4, Continuous),
            Param::MacroZ => ("Macro Z", "Z", 0.4, Continuous),
            Param::Level1 => ("Chamber 1 level", "LEVEL", 1.0, Continuous),
            Param::Tune2 => ("Chamber 2 tune", "TUNE", 0.25, Continuous),
            Param::Level2 => ("Chamber 2 level", "LEVEL", 0.0, Continuous),
            Param::Length2 => ("Chamber 2 length", "LENGTH", 0.6, Continuous),
            Param::Delay2 => ("Chamber 2 delay", "DELAY", 0.0, Continuous),
            Param::Tune3 => ("Chamber 3 tune", "TUNE", 0.75, Continuous),
            Param::Level3 => ("Chamber 3 level", "LEVEL", 0.0, Continuous),
            Param::Length3 => ("Chamber 3 length", "LENGTH", 0.6, Continuous),
            Param::Delay3 => ("Chamber 3 delay", "DELAY", 0.0, Continuous),
            Param::PeriRate => ("Peristalsis rate", "RATE", 0.4, Continuous),
            Param::PeriSqueeze => ("Peristalsis squeeze", "SQUEEZE", 0.0, Continuous),
            Param::PeriPitch => ("Peristalsis pitch", "PITCH", 0.5, Continuous),
            Param::PeriIrreg => ("Peristalsis irregularity", "IRREG", 0.0, Continuous),
            Param::PeriTravel => ("Peristalsis travel", "TRAVEL", 0.3, Continuous),
            Param::Attack => ("Attack", "ATTACK", 0.1, Continuous),
            Param::Curve => ("Attack curve", "CURVE", 0.5, Continuous),
            Param::Hold => ("Hold", "HOLD", 0.15, Continuous),
            Param::FizzLevel => ("Fizz level", "LEVEL", 0.0, Continuous),
            Param::FizzTone => ("Fizz tone", "TONE", 0.55, Continuous),
            Param::FizzCrackle => ("Fizz crackle", "CRACKLE", 0.3, Continuous),
            Param::SweepSpeed => ("Sweep time", "TIME", 0.35, Continuous),
            Param::SweepDepth => ("Sweep depth", "DEPTH", 0.5, Continuous),
            Param::ChaosRate => ("Chaos rate", "RATE", 0.4, Continuous),
            Param::ChaosDepth => ("Chaos depth", "DEPTH", 0.0, Continuous),
            Param::ChaosDest => (
                "Chaos destination",
                "DEST",
                CHAOS_PITCH as f32,
                Choice(CHAOS_DEST_NAMES),
            ),
            Param::Jitter => ("Per-hit jitter", "JITTER", 0.0, Continuous),
            Param::Cutoff => ("Filter cutoff", "CUTOFF", 1.0, Continuous),
            Param::Reso => ("Filter resonance", "RESO", 0.0, Continuous),
            Param::FilterEnv => ("Filter envelope", "ENV", 0.5, Continuous),
            Param::FilterType => (
                "Filter type",
                "TYPE",
                FILTER_LP24 as f32,
                Choice(FILTER_TYPE_NAMES),
            ),
            Param::Drive => ("Saturator drive", "DRIVE", 0.0, Continuous),
            Param::SatType => (
                "Saturator curve",
                "CURVE",
                SAT_TAPE as f32,
                Choice(SAT_TYPE_NAMES),
            ),
            Param::SatMix => ("Saturator mix", "MIX", 1.0, Continuous),
            Param::SatPos => (
                "Saturator position",
                "POS",
                SAT_PRE as f32,
                Choice(SAT_POS_NAMES),
            ),
            Param::LowCut => ("EQ low cut", "LO CUT", 0.0, Continuous),
            Param::LowFreq => ("EQ low shelf frequency", "LOW F", 0.4, Continuous),
            Param::LowGain => ("EQ low shelf gain", "LOW", 0.5, Continuous),
            Param::MidFreq => ("EQ mid frequency", "MID F", 0.45, Continuous),
            Param::MidGain => ("EQ mid gain", "MID", 0.5, Continuous),
            Param::MidQ => ("EQ mid Q", "Q", 0.35, Continuous),
            Param::HighFreq => ("EQ high shelf frequency", "HIGH F", 0.55, Continuous),
            Param::HighGain => ("EQ high shelf gain", "HIGH", 0.5, Continuous),
            Param::ChorusMix => ("Chorus amount", "AMOUNT", 0.0, Continuous),
            Param::ChorusRate => ("Chorus rate", "RATE", 0.45, Continuous),
            Param::ChorusDepth => ("Chorus depth", "DEPTH", 0.5, Continuous),
            Param::ChorusXover => ("Chorus crossover", "X-OVER", 0.3, Continuous),
            Param::Level => ("Output", "OUTPUT", 0.75, Continuous),
            Param::Sense => ("Sense", "SENSE", 0.5, Continuous),
            Param::Pan => ("Pan", "PAN", 0.5, Continuous),
            Param::ReverbMix => ("Reverb amount", "AMOUNT", 0.25, Continuous),
            Param::ReverbDecay => ("Reverb decay", "DECAY", 0.35, Continuous),
            Param::ReverbTone => ("Reverb tone", "TONE", 0.4, Continuous),
            Param::ReverbPredelay => ("Reverb pre-delay", "PRE", 0.1, Continuous),
        };
        ParamInfo {
            param: self,
            name,
            label,
            default,
            kind,
        }
    }

    pub fn default_value(self) -> f32 {
        self.info().default
    }

    /// Clamp/quantize a raw value to what this parameter accepts.
    pub fn sanitize(self, v: f32) -> f32 {
        let v = if v.is_finite() {
            v
        } else {
            self.default_value()
        };
        match self.info().kind {
            ParamKind::Continuous => v.clamp(0.0, 1.0),
            ParamKind::Choice(names) => v.round().clamp(0.0, (names.len() - 1) as f32),
        }
    }

    /// Bipolar controls are centred at 0.5 (the panel lights them from the
    /// middle). Macro knobs depend on the engine: see [`Param::is_bipolar_for`].
    pub fn is_bipolar(self) -> bool {
        matches!(
            self,
            Param::Fine
                | Param::Tune2
                | Param::Tune3
                | Param::PeriPitch
                | Param::Curve
                | Param::SweepDepth
                | Param::FilterEnv
                | Param::LowGain
                | Param::MidGain
                | Param::HighGain
                | Param::Pan
        )
    }

    pub fn is_bipolar_for(self, engine: u32) -> bool {
        match self.macro_slot() {
            Some(slot) => macro_bipolar(engine, slot),
            None => self.is_bipolar(),
        }
    }

    /// The macro slot (0 = X, 1 = Y, 2 = Z) of a macro parameter.
    pub fn macro_slot(self) -> Option<usize> {
        match self {
            Param::MacroX => Some(0),
            Param::MacroY => Some(1),
            Param::MacroZ => Some(2),
            _ => None,
        }
    }

    /// Controls that RND and MUTATE never change (and that therefore can't be
    /// locked): the channel's level, trigger sensitivity and stereo placement.
    pub fn is_performance(self) -> bool {
        matches!(self, Param::Level | Param::Sense | Param::Pan)
    }

    /// Panel label, resolving macro knobs to the given engine's names.
    pub fn label_for(self, engine: u32) -> &'static str {
        match self.macro_slot() {
            Some(slot) => macro_label(engine, slot),
            None => self.info().label,
        }
    }

    /// Descriptive name, resolving macro knobs to the given engine's names.
    pub fn name_for(self, engine: u32) -> &'static str {
        match self.macro_slot() {
            Some(slot) => macro_name(engine, slot),
            None => self.info().name,
        }
    }

    /// Human readable value for the panel read-out. Macro knobs need the
    /// engine; use [`Param::display_for`] for those.
    pub fn display(self, v: f32) -> String {
        self.display_for(ENGINE_GURGLE, v)
    }

    /// Human readable value, resolving macro knobs for the given engine.
    pub fn display_for(self, engine: u32, v: f32) -> String {
        if let Some(slot) = self.macro_slot() {
            return macro_display(engine, slot, v);
        }
        let off_or = |f: &dyn Fn() -> String| if v <= 0.0 { "OFF".into() } else { f() };
        match self {
            Param::Engine
            | Param::ChaosDest
            | Param::FilterType
            | Param::SatType
            | Param::SatPos => match self.info().kind {
                ParamKind::Choice(names) => names[self.sanitize(v) as usize].to_string(),
                ParamKind::Continuous => unreachable!(),
            },
            Param::Pitch => fmt_hz(map::pitch_hz(v, 0.5)),
            Param::Fine => format!("{:+.0} ct", map::fine_semitones(v) * 100.0),
            Param::Decay => fmt_secs(map::decay_seconds(v)),
            Param::Tune2 | Param::Tune3 => fmt_semitones(map::tune_semitones(v)),
            Param::Level1 | Param::Level2 | Param::Level3 => {
                let g = map::osc_gain(v);
                if g <= 0.0 { "OFF".into() } else { fmt_db(g) }
            }
            Param::Length2 | Param::Length3 => format!("x{:.2}", map::length_ratio(v)),
            Param::Delay2 | Param::Delay3 => off_or(&|| fmt_secs(map::chamber_delay_seconds(v))),
            Param::PeriRate => fmt_hz(map::peri_hz(v)),
            Param::PeriSqueeze | Param::PeriIrreg | Param::FizzCrackle => {
                off_or(&|| format!("{:.0}%", v * 100.0))
            }
            Param::PeriPitch => {
                let st = map::peri_semitones(v);
                if st.abs() < 0.05 {
                    "OFF".into()
                } else {
                    format!("{st:+.1} st")
                }
            }
            Param::PeriTravel => format!("{:.0}°", map::peri_travel_cycles(v) * 360.0),
            Param::Attack => fmt_secs(map::attack_seconds(v)),
            Param::Curve => {
                let b = (v - 0.5) * 200.0;
                if b.abs() < 1.0 {
                    "LINEAR".into()
                } else if b < 0.0 {
                    format!("SWELL {:.0}%", -b)
                } else {
                    format!("SNAP {:.0}%", b)
                }
            }
            Param::Hold => off_or(&|| fmt_secs(map::hold_seconds(v))),
            Param::FizzLevel => off_or(&|| fmt_db(map::fizz_gain(v))),
            Param::FizzTone => fmt_hz(map::fizz_hz(v)),
            Param::ChaosDepth | Param::Jitter | Param::ChorusMix => {
                off_or(&|| format!("{:.0}%", v * 100.0))
            }
            Param::SatMix => format!("{:.0}%", v * 100.0),
            Param::SweepSpeed => fmt_secs(map::sweep_seconds(v)),
            Param::SweepDepth => {
                let o = map::sweep_octaves(v);
                if o.abs() < 0.005 {
                    "OFF".into()
                } else {
                    format!("{:+.2} oct", o)
                }
            }
            Param::ChaosRate => fmt_hz(map::chaos_hz(v)),
            Param::Cutoff => fmt_hz(map::cutoff_hz(v)),
            Param::Reso => format!("{:.0}%", 100.0 * map::reso_k(v) / 4.0),
            Param::FilterEnv => format!("{:+.1} oct", map::filter_env_octaves(v)),
            Param::Drive => off_or(&|| format!("+{:.1} dB", map::drive_db(v))),
            Param::LowCut => off_or(&|| fmt_hz(map::low_cut_hz(v))),
            Param::LowFreq => fmt_hz(map::low_shelf_hz(v)),
            Param::MidFreq => fmt_hz(map::mid_hz(v)),
            Param::HighFreq => fmt_hz(map::high_shelf_hz(v)),
            Param::LowGain | Param::MidGain | Param::HighGain => {
                let db = map::eq_gain_db(v);
                if db.abs() < 0.05 {
                    "0 dB".into()
                } else {
                    format!("{db:+.1} dB")
                }
            }
            Param::MidQ => format!("Q {:.1}", map::mid_q(v)),
            Param::ChorusRate => fmt_hz(map::chorus_hz(v)),
            Param::ChorusDepth => format!("{:.1} ms", map::chorus_depth_ms(v)),
            Param::ChorusXover => off_or(&|| fmt_hz(map::chorus_xover_hz(v))),
            Param::Level => fmt_db(map::level_gain(v)),
            Param::Sense => format!("x{:.2}", map::sense_gain(v)),
            Param::Pan => {
                let p = (v - 0.5) * 200.0;
                if p.abs() < 1.0 {
                    "C".to_string()
                } else if p < 0.0 {
                    format!("L{:.0}", -p)
                } else {
                    format!("R{:.0}", p)
                }
            }
            Param::ReverbMix => fmt_db(map::reverb_gain(v)),
            Param::ReverbDecay => fmt_secs(map::reverb_seconds(v)),
            Param::ReverbTone => fmt_hz(map::reverb_tone_hz(v)),
            Param::ReverbPredelay => fmt_secs(map::reverb_predelay_seconds(v)),
            Param::MacroX | Param::MacroY | Param::MacroZ => unreachable!(),
        }
    }
}

/// Short panel label of an engine macro knob.
pub fn macro_label(engine: u32, slot: usize) -> &'static str {
    const LABELS: [[&str; 3]; ENGINE_COUNT] = [
        ["DENSITY", "SCATTER", "CHIRP"],
        ["TIGHT", "WET", "CHEEK"],
        ["VOWEL", "ROUGH", "SIZE"],
        ["PRESS", "SLIDE", "MEMBR"],
        ["SQUEEZE", "WAVE", "GAS"],
        ["RATIO", "COUPLE", "FEEDBK"],
        ["DENSITY", "CHIRP", "CHATTER"],
        ["WANDER", "ROUGH", "GRIT"],
    ];
    LABELS[(engine as usize).min(ENGINE_COUNT - 1)][slot.min(2)]
}

/// Descriptive name of an engine macro knob.
pub fn macro_name(engine: u32, slot: usize) -> &'static str {
    const NAMES: [[&str; 3]; ENGINE_COUNT] = [
        ["Bubble density", "Bubble size scatter", "Bubble chirp"],
        ["Sphincter tightness", "Wetness (sputter)", "Cheek cavity"],
        ["Vowel", "Roughness (jitter, subharmonics)", "Creature size"],
        ["Contact pressure", "Slide speed", "Membrane"],
        ["Constriction depth", "Peristaltic wave", "Buzz ↔ gas"],
        ["Partner ratio", "Coupling", "Self-feedback"],
        ["Critter density", "Chirp and warble", "Chatter (bursts)"],
        ["Pitch wander", "Roughness", "Grit (drive)"],
    ];
    NAMES[(engine as usize).min(ENGINE_COUNT - 1)][slot.min(2)]
}

/// Macro knobs that are centred (GURGLE's CHIRP and TRACT's WAVE).
pub fn macro_bipolar(engine: u32, slot: usize) -> bool {
    matches!((engine, slot), (ENGINE_GURGLE, 2) | (ENGINE_TRACT, 1))
}

/// Read-out of an engine macro knob.
pub fn macro_display(engine: u32, slot: usize, v: f32) -> String {
    let pct = || format!("{:.0}%", v * 100.0);
    match (engine, slot) {
        (ENGINE_GURGLE, 0) => format!("{:.1}/s", map::gurgle_density(v)),
        (ENGINE_GURGLE, 1) => format!("±{:.2} oct", map::gurgle_scatter_octaves(v)),
        (ENGINE_GURGLE, 2) => {
            let o = map::gurgle_chirp_octaves(v);
            if o.abs() < 0.005 {
                "FLAT".into()
            } else {
                format!("{o:+.2} oct")
            }
        }
        (ENGINE_FLATUS, 2) => fmt_hz(map::cheek_hz(v)),
        (ENGINE_GROWL, 0) => morph_name(&VOWEL_NAMES, v),
        (ENGINE_GROWL, 2) => format!("x{:.2}", map::growl_size(v)),
        (ENGINE_SQUEAK, 2) => morph_name(&MEMBRANE_NAMES, v),
        (ENGINE_TRACT, 1) => {
            let hz = map::tract_wave_hz(v);
            if hz.abs() < 0.005 {
                "STILL".into()
            } else {
                format!("{hz:+.2} Hz")
            }
        }
        (ENGINE_TRACT, 2) => morph_name(&GAS_NAMES, v),
        (ENGINE_SYMBIO, 0) => format!("x{:.3}", map::symbio_ratio(v)),
        (ENGINE_SYMBIO, 1) => format!("{:.2}", map::symbio_index(v)),
        (ENGINE_FLORA, 0) => format!("{:.1}/s", map::flora_density(v)),
        (ENGINE_FLORA, 1) => format!("±{:.2} oct", map::flora_chirp_octaves(v)),
        (ENGINE_RUMBLE, 0) => format!("±{:.1} st", map::rumble_wander_semitones(v)),
        (ENGINE_RUMBLE, 2) => format!("+{:.0} dB", map::rumble_grit_db(v)),
        _ => pct(),
    }
}

/// "OO", "OO>OH 40%" … for knobs that morph through named shapes.
fn morph_name(names: &[&str], v: f32) -> String {
    let (i, f) = map::segment(v, names.len());
    if f < 0.03 {
        names[i].to_string()
    } else if f > 0.97 {
        names[i + 1].to_string()
    } else {
        let short = |s: &str| s.chars().take(3).collect::<String>();
        format!(
            "{}>{} {:.0}%",
            short(names[i]),
            short(names[i + 1]),
            f * 100.0
        )
    }
}

fn fmt_hz(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{:.2} kHz", hz / 1000.0)
    } else if hz >= 100.0 {
        format!("{:.0} Hz", hz)
    } else if hz >= 1.0 {
        format!("{:.1} Hz", hz)
    } else {
        format!("{:.2} Hz", hz)
    }
}

fn fmt_db(gain: f32) -> String {
    if gain <= 1e-4 {
        "-inf dB".to_string()
    } else {
        format!("{:.1} dB", 20.0 * gain.log10())
    }
}

fn fmt_secs(s: f32) -> String {
    if s >= 10.0 {
        format!("{:.1} s", s)
    } else if s >= 1.0 {
        format!("{:.2} s", s)
    } else if s >= 0.01 {
        format!("{:.0} ms", s * 1000.0)
    } else {
        format!("{:.1} ms", s * 1000.0)
    }
}

fn fmt_semitones(st: f32) -> String {
    if st.abs() < 0.005 {
        "0.00 st".into()
    } else {
        format!("{st:+.2} st")
    }
}

/// Knob position → physical unit mappings.
pub mod map {
    /// Lowest pitch at the bottom of the PITCH fader.
    pub const PITCH_MIN_HZ: f32 = 16.0;
    /// Span of the PITCH fader in octaves (16 Hz … 4 kHz).
    pub const PITCH_OCTAVES: f32 = 8.0;
    /// Range of the chamber 2/3 TUNE knobs, ± semitones.
    pub const TUNE_RANGE_ST: f32 = 24.0;

    #[inline]
    fn bipolar(v: f32) -> f32 {
        (v - 0.5) * 2.0
    }

    /// `(segment index, fraction)` of `v` over `n` evenly spaced named points.
    pub fn segment(v: f32, n: usize) -> (usize, f32) {
        let s = v.clamp(0.0, 1.0) * (n - 1) as f32;
        let i = (s as usize).min(n - 2);
        (i, s - i as f32)
    }

    pub fn fine_semitones(fine: f32) -> f32 {
        bipolar(fine) * 2.0
    }

    pub fn pitch_octaves(pitch: f32, fine: f32) -> f32 {
        PITCH_MIN_HZ.log2() + pitch * PITCH_OCTAVES + fine_semitones(fine) / 12.0
    }

    pub fn pitch_hz(pitch: f32, fine: f32) -> f32 {
        pitch_octaves(pitch, fine).exp2()
    }

    /// PITCH fader position of a frequency (for presets and tests).
    pub fn pitch_knob(hz: f32) -> f32 {
        (hz / PITCH_MIN_HZ).log2() / PITCH_OCTAVES
    }

    /// Chamber 2/3 offset from chamber 1, in (fractional) semitones.
    pub fn tune_semitones(tune: f32) -> f32 {
        bipolar(tune) * TUNE_RANGE_ST
    }

    /// Knob position of a TUNE offset in semitones (for presets and tests).
    pub const fn tune_knob(semitones: f32) -> f32 {
        0.5 + semitones / (2.0 * TUNE_RANGE_ST)
    }

    /// Chamber level knob → gain.
    pub fn osc_gain(level: f32) -> f32 {
        level * level
    }

    /// -60 dB decay time of chamber 1: 30 ms … 20 s.
    pub fn decay_seconds(decay: f32) -> f32 {
        0.03 * (20.0f32 / 0.03).powf(decay)
    }

    /// Chamber 2/3 envelope length as a multiple of chamber 1's: ×0.125 …
    /// ×4, ×1 at 0.6.
    pub fn length_ratio(v: f32) -> f32 {
        0.125 * 32f32.powf(v)
    }

    /// Chamber 2/3 onset delay: 0 … 3 s.
    pub fn chamber_delay_seconds(v: f32) -> f32 {
        3.0 * v * v
    }

    /// Attack: 1 ms … 10 s.
    pub fn attack_seconds(attack: f32) -> f32 {
        0.001 * 10_000f32.powf(attack)
    }

    /// Exponent of the attack curve `t^p`: 4 (a slow swell that blooms at the
    /// end) … 1 (linear) … 0.25 (a fast snap that eases into the peak).
    pub fn attack_exponent(curve: f32) -> f32 {
        4f32.powf(-bipolar(curve))
    }

    /// Hold: 0 … 10 s.
    pub fn hold_seconds(hold: f32) -> f32 {
        10.0 * hold * hold
    }

    /// Peristalsis contraction rate: 0.1 … 25 Hz. Slow waves at the bottom,
    /// a fluttering "brrr" at the top.
    pub fn peri_hz(rate: f32) -> f32 {
        0.1 * 250f32.powf(rate)
    }

    /// Peristalsis pitch rise at the peak of a contraction: ±12 semitones.
    pub fn peri_semitones(v: f32) -> f32 {
        let x = bipolar(v);
        12.0 * x * x.abs()
    }

    /// Peristalsis lag from one chamber to the next, in cycles (0 … ½).
    pub fn peri_travel_cycles(v: f32) -> f32 {
        0.5 * v
    }

    /// Fizz layer gain.
    pub fn fizz_gain(level: f32) -> f32 {
        1.4 * level * level
    }

    /// Fizz band centre: 200 Hz … 12 kHz.
    pub fn fizz_hz(tone: f32) -> f32 {
        200.0 * 60f32.powf(tone)
    }

    /// Pitch sweep time constant: 2 ms … 10 s.
    pub fn sweep_seconds(speed: f32) -> f32 {
        0.002 * 5_000f32.powf(speed)
    }

    /// Bipolar sweep depth in octaves: above centre the hit starts high and
    /// falls back to the pitch, below it starts low and rises.
    pub fn sweep_octaves(depth: f32) -> f32 {
        let x = bipolar(depth);
        6.0 * x * x.abs().sqrt()
    }

    /// Approximate orbit rate of the chaos modulator.
    pub fn chaos_hz(rate: f32) -> f32 {
        0.05 * 600f32.powf(rate)
    }

    pub fn cutoff_hz(cutoff: f32) -> f32 {
        20.0 * 1000f32.powf(cutoff)
    }

    /// Ladder feedback; the filter self-oscillates just below the top.
    pub fn reso_k(reso: f32) -> f32 {
        4.3 * reso
    }

    pub fn filter_env_octaves(env: f32) -> f32 {
        let x = bipolar(env);
        8.0 * x * x.abs().sqrt()
    }

    pub fn drive_db(drive: f32) -> f32 {
        36.0 * drive
    }

    /// EQ low cut frequency (the knob at zero bypasses it).
    pub fn low_cut_hz(v: f32) -> f32 {
        10.0 * 40f32.powf(v)
    }

    pub fn low_shelf_hz(v: f32) -> f32 {
        20.0 * 40f32.powf(v)
    }

    pub fn mid_hz(v: f32) -> f32 {
        25.0 * 320f32.powf(v)
    }

    pub fn mid_q(v: f32) -> f32 {
        0.3 * 30f32.powf(v)
    }

    pub fn high_shelf_hz(v: f32) -> f32 {
        800.0 * 20f32.powf(v)
    }

    pub fn eq_gain_db(v: f32) -> f32 {
        bipolar(v) * 18.0
    }

    pub fn chorus_hz(rate: f32) -> f32 {
        0.05 * 160f32.powf(rate)
    }

    pub fn chorus_depth_ms(depth: f32) -> f32 {
        5.0 * depth
    }

    pub fn chorus_xover_hz(v: f32) -> f32 {
        20.0 * 30f32.powf(v)
    }

    pub fn level_gain(level: f32) -> f32 {
        level * level
    }

    pub fn sense_gain(sense: f32) -> f32 {
        0.25 * 16f32.powf(sense)
    }

    pub fn reverb_gain(amount: f32) -> f32 {
        amount * amount
    }

    /// Approximate RT60 of the plate.
    pub fn reverb_seconds(decay: f32) -> f32 {
        0.3 * 40f32.powf(decay)
    }

    /// Damping lowpass cutoff (input bandwidth and tank damping).
    pub fn reverb_tone_hz(tone: f32) -> f32 {
        1500.0 * 12f32.powf(tone)
    }

    pub fn reverb_predelay_seconds(predelay: f32) -> f32 {
        0.15 * predelay
    }

    // --- engine macros ---------------------------------------------------

    /// GURGLE DENSITY: bubbles per second at full pressure, 0.5 … 400.
    pub fn gurgle_density(v: f32) -> f32 {
        0.5 * 800f32.powf(v)
    }

    /// GURGLE SCATTER: spread of the bubble sizes (pitches), ± octaves.
    pub fn gurgle_scatter_octaves(v: f32) -> f32 {
        2.0 * v
    }

    /// GURGLE CHIRP: how far a bubble glides over its life. Down to -1
    /// octave (sinking "bloop"), up to +2 (a rising "plink").
    pub fn gurgle_chirp_octaves(v: f32) -> f32 {
        let x = bipolar(v);
        if x < 0.0 { x } else { 2.0 * x }
    }

    /// FLATUS TIGHT: the valve's nonlinearity (Van der Pol μ): a soft, round
    /// puff at the bottom, a tight relaxation buzz at the top.
    pub fn flatus_mu(v: f32) -> f32 {
        0.25 + 5.75 * v * v
    }

    /// FLATUS CHEEK: centre of the cavity resonance, 60 Hz … 2.4 kHz.
    pub fn cheek_hz(v: f32) -> f32 {
        60.0 * 40f32.powf(v)
    }

    /// GROWL SIZE: formant scale, ×0.35 (a vast beast) … ×1.6 (a critter).
    pub fn growl_size(v: f32) -> f32 {
        0.35 * (1.6f32 / 0.35).powf(v)
    }

    /// TRACT WAVE: speed and direction of the peristaltic wave, ±6 Hz.
    pub fn tract_wave_hz(v: f32) -> f32 {
        let x = bipolar(v);
        6.0 * x * x.abs()
    }

    /// SYMBIO RATIO: the partner's frequency ratio, 0.5 … 7.13.
    pub fn symbio_ratio(v: f32) -> f32 {
        0.5 * (7.13f32 / 0.5).powf(v)
    }

    /// SYMBIO COUPLE: peak coupling index (radians).
    pub fn symbio_index(v: f32) -> f32 {
        8.0 * v * v
    }

    /// FLORA DENSITY: critters per second at full pressure, 1 … 300.
    pub fn flora_density(v: f32) -> f32 {
        300f32.powf(v)
    }

    /// FLORA CHIRP: how far each critter glides, ± octaves.
    pub fn flora_chirp_octaves(v: f32) -> f32 {
        3.0 * v
    }

    /// RUMBLE WANDER: depth of the partials' pitch drift, ± semitones.
    pub fn rumble_wander_semitones(v: f32) -> f32 {
        12.0 * v * v
    }

    /// RUMBLE GRIT: drive into the shaper.
    pub fn rumble_grit_db(v: f32) -> f32 {
        24.0 * v
    }
}

/// A complete voice patch: one value per [`Param`], indexed by `Param::index`.
pub type VoicePatch = [f32; PARAM_COUNT];

pub fn default_patch() -> VoicePatch {
    let mut p = [0.0; PARAM_COUNT];
    for param in ALL_PARAMS {
        p[param.index()] = param.default_value();
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        for (i, p) in ALL_PARAMS.iter().enumerate() {
            assert_eq!(p.index(), i);
            assert_eq!(Param::from_id(p.id()), Some(*p));
        }
        assert_eq!(Param::from_id(PARAM_COUNT as u32), None);
    }

    #[test]
    fn sanitize_quantizes_choices() {
        assert_eq!(Param::Engine.sanitize(2.4), 2.0);
        assert_eq!(Param::Engine.sanitize(99.0), 7.0);
        assert_eq!(Param::FilterType.sanitize(9.0), 3.0);
        assert_eq!(Param::Pitch.sanitize(1.5), 1.0);
        assert_eq!(
            Param::Pitch.sanitize(f32::NAN),
            Param::Pitch.default_value()
        );
    }

    #[test]
    fn mapping_ranges() {
        assert!((map::pitch_hz(0.0, 0.5) - 16.0).abs() < 0.01);
        assert!((map::pitch_hz(1.0, 0.5) - 4096.0).abs() < 1.0);
        assert!((map::pitch_knob(440.0) - 0.597).abs() < 0.001);
        assert!((map::sweep_seconds(0.0) - 0.002).abs() < 1e-5);
        assert!((map::sweep_seconds(1.0) - 10.0).abs() < 1e-2);
        assert!((map::decay_seconds(1.0) - 20.0).abs() < 1e-2);
        assert!((map::attack_seconds(1.0) - 10.0).abs() < 1e-2);
        assert!((map::attack_seconds(0.0) - 0.001).abs() < 1e-6);
        assert!((map::hold_seconds(1.0) - 10.0).abs() < 1e-4);
        assert!((map::length_ratio(0.6) - 1.0).abs() < 1e-4);
        assert!((map::attack_exponent(0.5) - 1.0).abs() < 1e-6);
        assert!(map::attack_exponent(0.0) > 3.9 && map::attack_exponent(1.0) < 0.26);
        assert!((map::cutoff_hz(1.0) - 20_000.0).abs() < 1.0);
        assert_eq!(map::sweep_octaves(0.5), 0.0);
        assert_eq!(map::tune_semitones(0.5), 0.0);
        assert_eq!(map::peri_semitones(0.5), 0.0);
        assert_eq!(map::gurgle_chirp_octaves(0.5), 0.0);
        assert_eq!(map::tract_wave_hz(0.5), 0.0);
        assert!((map::tune_semitones(map::tune_knob(-12.0)) + 12.0).abs() < 1e-4);
        assert!((map::tune_semitones(map::tune_knob(7.02)) - 7.02).abs() < 1e-4);
        assert_eq!(map::eq_gain_db(0.5), 0.0);
        assert_eq!(map::segment(1.0, 8), (6, 1.0));
    }

    #[test]
    fn display_is_sane() {
        assert_eq!(Param::Engine.display(1.0), "FLATUS");
        assert_eq!(Param::Pan.display(0.5), "C");
        assert_eq!(Param::Pitch.display(1.0), "4.10 kHz");
        assert_eq!(Param::SweepDepth.display(0.5), "OFF");
        assert_eq!(Param::Tune2.display(0.25), "-12.00 st");
        assert_eq!(Param::LowGain.display(0.5), "0 dB");
        assert_eq!(Param::LowCut.display(0.0), "OFF");
        assert_eq!(Param::FilterType.display(1.0), "LP24");
        assert_eq!(Param::Curve.display(0.5), "LINEAR");
        assert_eq!(Param::Curve.display(0.0), "SWELL 100%");
        assert_eq!(Param::Attack.display(1.0), "10.0 s");
        assert_eq!(Param::Delay2.display(0.0), "OFF");
        assert_eq!(Param::MacroX.display_for(ENGINE_GROWL, 0.0), "OO");
        assert_eq!(Param::MacroX.display_for(ENGINE_GROWL, 1.0), "EE");
        assert_eq!(Param::MacroZ.display_for(ENGINE_GURGLE, 0.5), "FLAT");
        assert_eq!(Param::MacroY.display_for(ENGINE_TRACT, 0.5), "STILL");
        assert_eq!(Param::MacroX.label_for(ENGINE_FLATUS), "TIGHT");
        assert_eq!(Param::Decay.label_for(ENGINE_RUMBLE), "DECAY");
    }

    #[test]
    fn every_macro_has_a_label_and_display() {
        for e in 0..ENGINE_COUNT as u32 {
            for slot in 0..3 {
                assert!(!macro_label(e, slot).is_empty());
                assert!(!macro_name(e, slot).is_empty());
                for v in [0.0, 0.33, 0.5, 1.0] {
                    assert!(!macro_display(e, slot, v).is_empty());
                }
            }
        }
    }

    #[test]
    fn every_param_displays() {
        for p in ALL_PARAMS {
            for v in [0.0, 0.5, 1.0] {
                assert!(!p.display(p.sanitize(v)).is_empty(), "{p:?}");
            }
        }
    }
}
