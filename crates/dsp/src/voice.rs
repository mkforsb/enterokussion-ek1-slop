//! One Enterokussion channel, up to the stereo stage.
//!
//! ```text
//!  PITCH ─┬─ sweep ─┬─ peristalsis ─┬─ chaos ─┬─ jitter ─► pitch ─┬─ +TUNE2 ─┬─ +TUNE3
//!         ▼         ▼               ▼         ▼                   ▼          ▼          ▼
//!                                                     ┌── CHAMBER 1 ── CHAMBER 2 ── CHAMBER 3 ──┐ (the engine, ×3)
//!  TRIGGER ─► envelopes (DELAY, ATTACK·CURVE, HOLD, DECAY, × LENGTH per chamber)                │
//!                ×  peristaltic squeeze  ═► pressure into each chamber, and its level ─────────┴─► Σ ─┐
//!            └─► FIZZ (hiss … crackle) × chamber 1's envelope ─────────────────────────────────────────┤
//!                                    ┌─────────────────────────────────────────────────────────────────┘
//!                                    ▼
//!                       [SAT pre] ─► LADDER ─► EQ ─► [SAT post] ─► OUTPUT ─► (chorus, pan, plate)
//! ```
//!
//! Each chamber has its own envelope: chamber 1 sets the times, chambers 2
//! and 3 stretch them (LENGTH) and can start late (DELAY). The peristaltic
//! squeeze multiplies each chamber's envelope, with a lag from one chamber
//! to the next, and the product is the chamber's *pressure*: it drives the
//! engine's process (see [`crate::engines`]) as well as its level.
//!
//! Controls are evaluated once per [`CONTROL_BLOCK`] samples (parameter
//! smoothing, modulation, engine and filter coefficients). The envelopes,
//! the squeeze (ramped across the block) and the audio path run every
//! sample.

use crate::engines::{Ctx, Engines, OSCS, OscCtx};
use crate::envelope::Env;
use crate::fizz::Fizz;
use crate::fx::{Eq, SatCoeffs};
use crate::modulation::{ChaosLfo, Peristalsis};
use crate::params::{
    CHAOS_CUTOFF, CHAOS_PITCH, CHAOS_X, CHAOS_Y, CHAOS_Z, PARAM_COUNT, Param, SAT_POST, VoicePatch,
    default_patch, map,
};
use crate::util::{Ladder, LadderCoeffs, Noise, one_pole_coeff};

/// Samples per control update.
pub const CONTROL_BLOCK: usize = 16;
/// Parameter de-zippering.
const SMOOTH_TAU_S: f32 = 0.004;
/// Chaos modulation range at full depth.
const CHAOS_PITCH_OCT: f32 = 1.0;
const CHAOS_MACRO: f32 = 0.5;
const CHAOS_CUTOFF_OCT: f32 = 4.0;
/// Jitter range at full setting.
const JITTER_PITCH_OCT: f32 = 0.5;
const JITTER_MACRO: f32 = 0.35;

/// Which parameters are de-zippered (the continuous ones).
const SMOOTHED: [bool; PARAM_COUNT] = {
    let mut s = [true; PARAM_COUNT];
    s[Param::Engine as usize] = false;
    s[Param::ChaosDest as usize] = false;
    s[Param::FilterType as usize] = false;
    s[Param::SatType as usize] = false;
    s[Param::SatPos as usize] = false;
    s
};

/// The chambers' (tune, level, length, delay) parameters. Chamber 1 is the
/// reference: no offset, the envelope as set, no delay.
#[allow(clippy::type_complexity)]
const CHAMBER_PARAMS: [(Option<Param>, Param, Option<Param>, Option<Param>); OSCS] = [
    (None, Param::Level1, None, None),
    (
        Some(Param::Tune2),
        Param::Level2,
        Some(Param::Length2),
        Some(Param::Delay2),
    ),
    (
        Some(Param::Tune3),
        Param::Level3,
        Some(Param::Length3),
        Some(Param::Delay3),
    ),
];

#[derive(Clone, Debug)]
pub struct Voice {
    fs: f32,
    target: VoicePatch,
    smooth: VoicePatch,
    k_smooth: f32,

    noise: Noise,
    fizz: Fizz,
    fizz_gain: f32,
    engines: Engines,
    engine: u32,
    ctx: Ctx,
    oscs: [OscCtx; OSCS],
    block_pos: usize,

    /// One envelope per chamber, `0..velocity`.
    env: [Env; OSCS],
    velocity: f32,
    /// Level × mix normalization per chamber.
    osc_gain: [f32; OSCS],
    /// Peristaltic squeeze per chamber, ramped across each block.
    squeeze: [f32; OSCS],
    squeeze_inc: [f32; OSCS],
    peristalsis: Peristalsis,
    /// Sweep envelope, 1 → 0.
    sweep: f32,
    sweep_mul: f32,

    chaos: ChaosLfo,
    /// Per-hit offsets: pitch (octaves), X, Y, Z.
    jitter: [f32; 4],

    ladder: Ladder,
    ladder_coeffs: LadderCoeffs,
    sat: SatCoeffs,
    sat_post: bool,
    eq: Eq,
    level: f32,
}

impl Voice {
    /// `fs` is the rate `tick` is called at (the oversampled rate).
    pub fn new(fs: f32, seed: u32) -> Self {
        let patch = default_patch();
        let mut v = Self {
            fs,
            target: patch,
            smooth: patch,
            k_smooth: one_pole_coeff(SMOOTH_TAU_S, fs / CONTROL_BLOCK as f32),
            noise: Noise::new(seed),
            fizz: Fizz::new(fs),
            fizz_gain: 0.0,
            engines: Engines::new(),
            engine: patch[Param::Engine.index()] as u32,
            ctx: Ctx::new(fs),
            oscs: [OscCtx::default(); OSCS],
            block_pos: 0,
            env: [Env::default(); OSCS],
            velocity: 1.0,
            osc_gain: [0.0; OSCS],
            squeeze: [1.0; OSCS],
            squeeze_inc: [0.0; OSCS],
            peristalsis: Peristalsis::new(seed),
            sweep: 0.0,
            sweep_mul: 0.0,
            chaos: ChaosLfo::new(seed),
            jitter: [0.0; 4],
            ladder: Ladder::default(),
            ladder_coeffs: LadderCoeffs::new(20_000.0, 0.0, 1, fs),
            sat: SatCoeffs::default(),
            sat_post: false,
            eq: Eq::default(),
            level: 0.0,
        };
        v.control();
        v
    }

    pub fn set_param(&mut self, param: Param, value: f32) {
        let v = param.sanitize(value);
        self.target[param.index()] = v;
        if !SMOOTHED[param.index()] {
            self.smooth[param.index()] = v;
        }
    }

    pub fn param(&self, param: Param) -> f32 {
        self.target[param.index()]
    }

    /// The de-zippered value the voice is currently using.
    pub fn smoothed(&self, param: Param) -> f32 {
        self.smooth[param.index()]
    }

    /// Jump all smoothed parameters to their targets.
    pub fn snap_params(&mut self) {
        self.smooth = self.target;
        self.control();
    }

    /// Fire the trigger. `velocity` is the raw hit strength `0..=1`; SENSE
    /// scales it like a trigger input's preamp.
    pub fn trigger(&mut self, velocity: f32) {
        let gain = map::sense_gain(self.target[Param::Sense.index()]);
        self.velocity = (velocity.clamp(0.0, 1.0) * gain).clamp(0.02, 1.0);
        let p = self.smooth;
        for (env, &(_, _, _, delay)) in self.env.iter_mut().zip(&CHAMBER_PARAMS) {
            let seconds = delay.map_or(0.0, |d| map::chamber_delay_seconds(p[d.index()]));
            env.trigger(self.velocity, (seconds * self.fs) as u32);
        }
        self.sweep = 1.0;
        self.peristalsis.trigger(p[Param::PeriIrreg.index()]);
        let amount = p[Param::Jitter.index()];
        self.jitter = if amount > 0.0 {
            [
                JITTER_PITCH_OCT * amount * amount * self.noise.sample(),
                JITTER_MACRO * amount * self.noise.sample(),
                JITTER_MACRO * amount * self.noise.sample(),
                JITTER_MACRO * amount * self.noise.sample(),
            ]
        } else {
            [0.0; 4]
        };
        // Evaluate the controls now so the engine sees this hit's pitch,
        // jitter and velocity from its first sample.
        self.control();
        self.block_pos = 1;
        self.engines
            .trigger(self.engine, &self.ctx, &self.oscs, &mut self.noise);
    }

    /// Chaos LED: lit on one lobe of the attractor while it modulates.
    pub fn lfo_led(&self) -> bool {
        self.smooth[Param::ChaosDepth.index()] > 0.0 && self.chaos.value() > 0.0
    }

    /// Current amplitude envelope of chamber 1 (for metering).
    pub fn envelope(&self) -> f32 {
        self.env[0].level()
    }

    /// The selected engine.
    pub fn engine(&self) -> u32 {
        self.engine
    }

    /// Control-rate update: smoothing, modulation, coefficients.
    fn control(&mut self) {
        let fs = self.fs;
        let block_s = CONTROL_BLOCK as f32 / fs;
        for ((cur, &target), &smoothed) in self.smooth.iter_mut().zip(&self.target).zip(&SMOOTHED) {
            if smoothed {
                *cur += (target - *cur) * self.k_smooth;
            }
        }
        let p = self.smooth;
        let param = |q: Param| p[q.index()];

        let engine = param(Param::Engine) as u32;
        if engine != self.engine {
            self.engines.reset(engine);
            self.engine = engine;
        }

        // Modulation.
        self.chaos.advance(param(Param::ChaosRate), block_s);
        let depth = param(Param::ChaosDepth);
        let chaos = self.chaos.value() * depth * depth;
        let dest = param(Param::ChaosDest) as u32;
        let chaos_to = |d: u32| if dest == d { chaos } else { 0.0 };
        let irregularity = param(Param::PeriIrreg);
        self.peristalsis
            .advance(map::peri_hz(param(Param::PeriRate)), irregularity, block_s);

        let macro_value =
            |q: Param, d: u32, j: f32| (param(q) + CHAOS_MACRO * chaos_to(d) + j).clamp(0.0, 1.0);
        self.ctx = Ctx {
            fs,
            x: macro_value(Param::MacroX, CHAOS_X, self.jitter[1]),
            y: macro_value(Param::MacroY, CHAOS_Y, self.jitter[2]),
            z: macro_value(Param::MacroZ, CHAOS_Z, self.jitter[3]),
            velocity: self.velocity,
        };

        // The chambers: envelope, squeeze, pitch, pressure and level.
        let decay_s = map::decay_seconds(param(Param::Decay));
        let attack_s = map::attack_seconds(param(Param::Attack));
        let hold_s = map::hold_seconds(param(Param::Hold));
        let exponent = map::attack_exponent(param(Param::Curve));
        let squeeze = param(Param::PeriSqueeze);
        // Squeezing takes level away between contractions; give some back.
        let makeup = 1.0 / (1.0 - 0.5 * squeeze);
        let travel = map::peri_travel_cycles(param(Param::PeriTravel));
        let peri_oct = map::peri_semitones(param(Param::PeriPitch)) / 12.0;
        let oct = map::pitch_octaves(param(Param::Pitch), param(Param::Fine))
            + map::sweep_octaves(param(Param::SweepDepth)) * self.sweep
            + CHAOS_PITCH_OCT * chaos_to(CHAOS_PITCH)
            + self.jitter[0];
        // Pressure follows the velocity too: a harder hit pushes more gas.
        let push = 0.35 + 0.65 * self.velocity;
        let mut gains = [0.0f32; OSCS];
        for (i, &(tune, level, length, _)) in CHAMBER_PARAMS.iter().enumerate() {
            let semis = tune.map_or(0.0, |t| map::tune_semitones(param(t)));
            let len = length.map_or(1.0, |l| map::length_ratio(param(l)));
            self.env[i].set_rates(attack_s * len, hold_s * len, decay_s * len, exponent, fs);
            let contraction = self.peristalsis.contraction(i as f32 * travel);
            let m = 1.0 - squeeze + squeeze * contraction;
            self.squeeze_inc[i] = (m - self.squeeze[i]) / CONTROL_BLOCK as f32;
            self.oscs[i] = OscCtx {
                hz: (oct + semis / 12.0 + peri_oct * contraction)
                    .exp2()
                    .clamp(2.0, 0.45 * fs),
                env: (self.env[i].level() / self.velocity * m * push).clamp(0.0, 1.0),
                decay_s: decay_s * len,
            };
            gains[i] = map::osc_gain(param(level));
        }
        // Stacking chambers shouldn't simply pile up level: scale the sum
        // back towards the loudness of one.
        let total: f32 = gains.iter().sum();
        let norm = makeup / total.max(1.0).powf(0.75);
        self.osc_gain = gains.map(|g| g * norm);
        self.engines.control(self.engine, &self.ctx, &self.oscs);

        self.fizz_gain = map::fizz_gain(param(Param::FizzLevel)) * makeup;
        if self.fizz_gain > 0.0 {
            self.fizz
                .update(param(Param::FizzTone), param(Param::FizzCrackle), fs);
        }

        self.sweep_mul = (-1.0 / (map::sweep_seconds(param(Param::SweepSpeed)) * fs)).exp();

        // Filter: the envelope moves the cutoff, the chaos can too.
        let cut_oct = map::cutoff_hz(param(Param::Cutoff)).log2()
            + map::filter_env_octaves(param(Param::FilterEnv)) * self.env[0].level()
            + CHAOS_CUTOFF_OCT * chaos_to(CHAOS_CUTOFF);
        self.ladder_coeffs = LadderCoeffs::new(
            cut_oct.exp2(),
            map::reso_k(param(Param::Reso)),
            param(Param::FilterType) as u32,
            fs,
        );

        self.sat = SatCoeffs::new(
            param(Param::Drive),
            param(Param::SatType) as u32,
            param(Param::SatMix),
        );
        self.sat_post = param(Param::SatPos) as u32 == SAT_POST;
        self.eq.update(
            [
                param(Param::LowCut),
                param(Param::LowFreq),
                param(Param::LowGain),
                param(Param::MidFreq),
                param(Param::MidGain),
                param(Param::MidQ),
                param(Param::HighFreq),
                param(Param::HighGain),
            ],
            fs,
        );
        self.level = map::level_gain(param(Param::Level));
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        if self.block_pos == 0 {
            self.control();
        }
        self.block_pos += 1;
        if self.block_pos == CONTROL_BLOCK {
            self.block_pos = 0;
        }

        let mut gains = [0.0f32; OSCS];
        for i in 0..OSCS {
            let e = self.env[i].tick();
            self.squeeze[i] += self.squeeze_inc[i];
            gains[i] = self.osc_gain[i] * e * self.squeeze[i];
        }
        self.sweep *= self.sweep_mul;
        if self.sweep < 1e-6 {
            self.sweep = 0.0;
        }

        let mut x = self.engines.tick(self.engine, &gains);
        if self.fizz_gain > 0.0 {
            let g = self.fizz_gain * self.env[0].level() * self.squeeze[0];
            if g > 0.0 {
                x += g * self.fizz.tick(&mut self.noise);
            }
        }
        if !x.is_finite() {
            // Never let a numerical accident poison the channel.
            self.engines.reset(self.engine);
            self.ladder.reset();
            self.fizz.reset();
            x = 0.0;
        }

        if self.sat.active() && !self.sat_post {
            x = self.sat.process(x);
        }
        x = self.ladder.process(x, &self.ladder_coeffs);
        x = self.eq.process(x);
        if self.sat.active() && self.sat_post {
            x = self.sat.process(x);
        }
        if !x.is_finite() {
            self.ladder.reset();
            self.eq = Eq::default();
            x = 0.0;
        }
        x * self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{ENGINE_COUNT, ENGINE_FLATUS, ENGINE_RUMBLE, ENGINE_SYMBIO, FILTER_HP};

    fn render(v: &mut Voice, n: usize) -> Vec<f32> {
        (0..n).map(|_| v.tick()).collect()
    }

    fn peak(x: &[f32]) -> f32 {
        x.iter().fold(0.0f32, |a, &b| a.max(b.abs()))
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
    }

    fn zero_crossings(x: &[f32]) -> usize {
        x.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count()
    }

    /// A plain sine at PITCH: SYMBIO with the partner in unison and no
    /// coupling, no sweep.
    fn sine_voice(fs: f32) -> Voice {
        let mut v = Voice::new(fs, 1);
        v.set_param(Param::Engine, ENGINE_SYMBIO as f32);
        v.set_param(Param::MacroX, 2f32.ln() / 14.26f32.ln());
        v.set_param(Param::MacroY, 0.0);
        v.set_param(Param::MacroZ, 0.0);
        v.set_param(Param::SweepDepth, 0.5);
        v.set_param(Param::Attack, 0.0);
        v.set_param(Param::Hold, 0.0);
        v.set_param(Param::Decay, 0.7);
        v
    }

    #[test]
    fn each_engine_sounds_and_decays() {
        let fs = 96_000.0;
        for engine in 0..ENGINE_COUNT {
            let mut v = Voice::new(fs, 1);
            v.set_param(Param::Engine, engine as f32);
            v.set_param(Param::Attack, 0.1);
            v.set_param(Param::Hold, 0.0);
            v.set_param(Param::Decay, 0.3);
            v.snap_params();
            v.trigger(1.0);
            let early = peak(&render(&mut v, 9_600));
            let _ = render(&mut v, fs as usize * 2);
            let late = peak(&render(&mut v, 9_600));
            assert!(early > 0.05, "engine {engine}: quiet ({early})");
            assert!(early < 2.0, "engine {engine}: loud ({early})");
            assert!(late < early * 0.01, "engine {engine}: {early} -> {late}");
        }
    }

    #[test]
    fn pitch_tracks_the_fader_and_tune_is_free() {
        let fs = 96_000.0;
        for (pitch, tune) in [(0.3f32, 0.0f32), (0.45, 7.02), (0.4, -12.0), (0.35, 3.33)] {
            let mut v = sine_voice(fs);
            v.set_param(Param::Pitch, pitch);
            // Chamber 2 alone, offset by `tune` semitones.
            v.set_param(Param::Level1, 0.0);
            v.set_param(Param::Level2, 1.0);
            v.set_param(Param::Length2, 1.0);
            v.set_param(Param::Tune2, map::tune_knob(tune));
            v.snap_params();
            v.trigger(1.0);
            let out = render(&mut v, fs as usize);
            let measured = zero_crossings(&out[9_600..]) as f32 / (1.0 - 0.1);
            let expected = map::pitch_hz(pitch, 0.5) * (tune / 12.0).exp2();
            let cents = 1200.0 * (measured / expected).log2();
            assert!(
                cents.abs() < 20.0,
                "{pitch} {tune}: {measured} vs {expected}"
            );
        }
    }

    /// The point of EK-1's envelopes: a 5 s swell that keeps growing.
    #[test]
    fn long_attack_swells() {
        let fs = 8_000.0;
        let mut v = sine_voice(fs);
        v.set_param(Param::Attack, 0.85); // ≈ 2.5 s
        v.set_param(Param::Decay, 0.8);
        v.snap_params();
        v.trigger(1.0);
        let out = render(&mut v, (fs * 3.0) as usize);
        let at = |s: f32| rms(&out[(s * fs) as usize..((s + 0.1) * fs) as usize]);
        assert!(
            at(0.2) < at(1.0) && at(1.0) < at(2.0),
            "{} {} {}",
            at(0.2),
            at(1.0),
            at(2.0)
        );
        assert!(at(0.2) < 0.2 * at(2.4));
    }

    #[test]
    fn swell_curve_holds_back_then_blooms() {
        let fs = 8_000.0;
        let early = |curve: f32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Attack, 0.75); // 1 s
            v.set_param(Param::Curve, curve);
            v.set_param(Param::Decay, 0.9);
            v.snap_params();
            v.trigger(1.0);
            let out = render(&mut v, (fs * 1.2) as usize);
            rms(&out[2_000..2_400]) / rms(&out[8_400..8_800])
        };
        assert!(
            early(0.0) < 0.3 * early(0.5),
            "{} {}",
            early(0.0),
            early(0.5)
        );
        assert!(early(1.0) > 1.5 * early(0.5));
    }

    /// A delayed chamber is silent until its onset.
    #[test]
    fn chamber_delay_holds_the_onset_back() {
        let fs = 8_000.0;
        let mut v = sine_voice(fs);
        v.set_param(Param::Level1, 0.0);
        v.set_param(Param::Level2, 1.0);
        v.set_param(Param::Delay2, 0.58); // ≈ 1 s
        v.snap_params();
        v.trigger(1.0);
        let out = render(&mut v, (fs * 1.5) as usize);
        assert!(peak(&out[..7_000]) < 1e-4);
        assert!(peak(&out[9_000..]) > 0.1);
    }

    #[test]
    fn chamber_length_stretches_the_envelope() {
        let fs = 8_000.0;
        let tail = |length: f32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Decay, 0.4);
            v.set_param(Param::Level1, 0.0);
            v.set_param(Param::Level2, 1.0);
            v.set_param(Param::Length2, length);
            v.snap_params();
            v.trigger(1.0);
            let out = render(&mut v, 8_000);
            rms(&out[4_000..]) / rms(&out[..800])
        };
        assert!(tail(1.0) > 4.0 * tail(0.6));
    }

    #[test]
    fn hold_keeps_the_body_up() {
        let fs = 8_000.0;
        let level_at_1s = |hold: f32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Decay, 0.3);
            v.set_param(Param::Hold, hold);
            v.snap_params();
            v.trigger(1.0);
            let out = render(&mut v, 9_000);
            peak(&out[8_000..])
        };
        assert!(level_at_1s(0.5) > 5.0 * level_at_1s(0.0));
    }

    /// Peristalsis pumps the level at its rate.
    #[test]
    fn peristalsis_squeezes_rhythmically() {
        let fs = 8_000.0;
        let mut v = sine_voice(fs);
        v.set_param(Param::Pitch, 0.4);
        v.set_param(Param::Hold, 0.6);
        v.set_param(Param::PeriSqueeze, 1.0);
        v.set_param(Param::PeriRate, map_rate(2.0));
        v.snap_params();
        v.trigger(1.0);
        let out = render(&mut v, (fs * 3.0) as usize);
        let env: Vec<f32> = out.as_chunks::<200>().0.iter().map(|w| rms(w)).collect();
        let (lo, hi) = env[4..]
            .iter()
            .fold((f32::MAX, 0.0f32), |(a, b), &e| (a.min(e), b.max(e)));
        assert!(lo < 0.1 * hi, "{lo} {hi}");
        // About six contractions in three seconds.
        let mut peaks = 0;
        for w in env.windows(3) {
            if w[1] > w[0] && w[1] >= w[2] && w[1] > 0.5 * hi {
                peaks += 1;
            }
        }
        assert!((5..=7).contains(&peaks), "{peaks}");
    }

    fn map_rate(hz: f32) -> f32 {
        (hz / 0.1).ln() / 250f32.ln()
    }

    /// The squeeze raises the pitch when PITCH is up.
    #[test]
    fn peristalsis_bends_the_pitch() {
        let fs = 16_000.0;
        let hz = |peri_pitch: f32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Pitch, 0.4);
            v.set_param(Param::Hold, 0.5);
            v.set_param(Param::PeriSqueeze, 0.0);
            v.set_param(Param::PeriRate, 0.0);
            v.set_param(Param::PeriPitch, peri_pitch);
            v.snap_params();
            v.trigger(1.0);
            // At the start the wave is on its way into a squeeze.
            zero_crossings(&render(&mut v, 4_000))
        };
        assert!(hz(1.0) > hz(0.5) + 5, "{} {}", hz(1.0), hz(0.5));
    }

    /// Velocity is pressure: FLATUS barely buzzes on a soft hit and blasts
    /// on a hard one, not just louder but brighter.
    #[test]
    fn velocity_is_pressure() {
        let fs = 48_000.0;
        let run = |vel: f32| {
            let mut v = Voice::new(fs, 3);
            v.set_param(Param::Engine, ENGINE_FLATUS as f32);
            v.set_param(Param::Sense, 0.5);
            v.set_param(Param::Hold, 0.4);
            v.snap_params();
            v.trigger(vel);
            render(&mut v, 24_000)[4_800..].to_vec()
        };
        let bright = |x: &[f32]| {
            let d = x.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f32>();
            (d / x.iter().map(|v| v * v).sum::<f32>()).sqrt()
        };
        let (soft, hard) = (run(0.3), run(1.0));
        assert!(rms(&hard) > 2.0 * rms(&soft));
        assert!(bright(&hard) > bright(&soft));
    }

    #[test]
    fn fizz_adds_a_layer() {
        let fs = 48_000.0;
        let level = |fizz: f32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Level1, 0.0);
            v.set_param(Param::FizzLevel, fizz);
            v.snap_params();
            v.trigger(1.0);
            rms(&render(&mut v, 9_600))
        };
        assert!(level(0.0) < 1e-6);
        assert!(level(0.8) > 0.05);
    }

    #[test]
    fn highpass_removes_the_low_end() {
        let fs = 48_000.0;
        let level = |ftype: u32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Pitch, 0.2);
            v.set_param(Param::FilterType, ftype as f32);
            v.set_param(Param::Cutoff, 0.5);
            v.snap_params();
            v.trigger(1.0);
            rms(&render(&mut v, 9_600)[2_400..])
        };
        assert!(level(FILTER_HP) < 0.05 * level(1));
    }

    #[test]
    fn jitter_makes_hits_differ() {
        let fs = 96_000.0;
        let mut v = sine_voice(fs);
        v.set_param(Param::Pitch, 0.4);
        v.set_param(Param::Jitter, 1.0);
        v.snap_params();
        let mut pitches = Vec::new();
        for _ in 0..4 {
            v.trigger(1.0);
            pitches.push(zero_crossings(&render(&mut v, 19_200)));
        }
        let (lo, hi) = (pitches.iter().min().unwrap(), pitches.iter().max().unwrap());
        assert!(*hi as f32 > *lo as f32 * 1.05, "{pitches:?}");
    }

    #[test]
    fn drive_adds_harmonics_but_not_much_level() {
        let fs = 48_000.0;
        let run = |drive: f32, engine: u32| {
            let mut v = sine_voice(fs);
            v.set_param(Param::Engine, engine as f32);
            v.set_param(Param::MacroX, 0.0);
            v.set_param(Param::MacroY, 0.0);
            v.set_param(Param::MacroZ, 0.0);
            v.set_param(Param::Drive, drive);
            v.snap_params();
            v.trigger(1.0);
            render(&mut v, 9_600)
        };
        let brightness = |x: &[f32]| {
            let d = x.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f32>();
            (d / x.iter().map(|v| v * v).sum::<f32>()).sqrt()
        };
        // A sine, and RUMBLE's three partials (already a little bright).
        for (engine, ratio) in [(ENGINE_SYMBIO, 2.0), (ENGINE_RUMBLE, 1.5)] {
            let (clean, dirty) = (run(0.0, engine), run(0.8, engine));
            assert!(
                brightness(&dirty) > ratio * brightness(&clean),
                "engine {engine}"
            );
            assert!(peak(&dirty) < 2.0 * peak(&clean), "engine {engine}");
        }
    }

    #[test]
    fn extremes_stay_finite() {
        let fs = 96_000.0;
        for engine in 0..ENGINE_COUNT {
            for &corner in &[0.0f32, 1.0] {
                let mut v = Voice::new(fs, 9);
                v.set_param(Param::Engine, engine as f32);
                for p in crate::params::ALL_PARAMS {
                    if matches!(p.info().kind, crate::params::ParamKind::Continuous) {
                        v.set_param(p, corner);
                    }
                }
                v.set_param(Param::Level, 1.0);
                v.set_param(Param::ChaosDepth, 1.0);
                v.set_param(Param::ChaosRate, 1.0);
                v.snap_params();
                v.trigger(1.0);
                // Every EQ band at +18 dB into a self-oscillating ladder is
                // loud by design; the engine's output stage bounds it.
                for s in render(&mut v, 48_000) {
                    assert!(
                        s.is_finite() && s.abs() < 64.0,
                        "engine {engine} corner {corner}: {s}"
                    );
                }
            }
        }
    }
}
