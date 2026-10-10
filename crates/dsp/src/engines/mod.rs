//! The eight synthesis engines, one alien digestive process each.
//!
//! Every engine is a *chamber type*: the voice runs three instances of the
//! selected one (CHAMBER 1–3), each at its own freely tuned pitch and with its
//! own envelope, and mixes them. All three share the engine's macro knobs.
//!
//! [`Ctx`] carries the per-block values every chamber shares (the macros
//! after chaos and jitter, the velocity), [`OscCtx`] the per-chamber ones
//! (pitch after sweep, peristalsis, chaos and jitter, and its *pressure*).
//! `control` recomputes coefficients once per control block and `tick`
//! renders one sample, before the voice applies the envelope.
//!
//! Pressure is the chamber's envelope times the peristaltic squeeze. The
//! process engines (FLATUS, SQUEAK, TRACT, GURGLE, FLORA) are *driven* by
//! it: a valve that only buzzes above a threshold, friction that only slips
//! when pushed, bubbles that only form while there's gas. So a long swell,
//! a sagging tail or a peristaltic squeeze changes what the process does,
//! not only how loud it is. Each engine keeps its own output roughly level
//! across pressure, so the voice's envelope still sets the loudness.

pub mod flatus;
pub mod flora;
pub mod growl;
pub mod gurgle;
pub mod rumble;
pub mod squeak;
pub mod symbio;
pub mod tract;

use crate::util::Noise;

/// Chambers per voice.
pub const OSCS: usize = 3;

/// Control values shared by the three chambers for one block.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// Rate `tick` is called at.
    pub fs: f32,
    /// Engine macros, `0..=1`, after chaos and jitter.
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Velocity of the current hit after SENSE, `0..=1`.
    pub velocity: f32,
}

impl Ctx {
    pub fn new(fs: f32) -> Self {
        Self {
            fs,
            x: 0.4,
            y: 0.4,
            z: 0.4,
            velocity: 1.0,
        }
    }
}

/// Control values for one chamber for one block.
#[derive(Clone, Copy, Debug)]
pub struct OscCtx {
    /// Pitch in Hz, after sweep, peristalsis, chaos and jitter.
    pub hz: f32,
    /// Pressure: this chamber's envelope relative to its peak, times the
    /// peristaltic squeeze, times a velocity factor (`0.35 … 1`), `0..=1`.
    pub env: f32,
    /// This chamber's -60 dB decay time in seconds.
    pub decay_s: f32,
}

impl Default for OscCtx {
    fn default() -> Self {
        Self {
            hz: 110.0,
            env: 0.0,
            decay_s: 0.5,
        }
    }
}

/// One chamber of an engine.
pub trait Osc: Clone + Default {
    /// Silence and clear state (engine switch, numerical accident).
    fn reset(&mut self);
    /// A new hit: restart per-hit state. `rng` seeds the chamber's own
    /// random stream, so every hit of a stochastic engine differs.
    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise);
    /// Recompute coefficients for the coming block.
    fn control(&mut self, c: &Ctx, o: &OscCtx);
    /// One sample, roughly within ±1, before the amplitude envelope.
    fn tick(&mut self) -> f32;
}

/// All engines, three chambers each. Only the selected engine runs.
#[derive(Clone, Debug, Default)]
pub struct Engines {
    gurgle: [gurgle::Gurgle; OSCS],
    flatus: [flatus::Flatus; OSCS],
    growl: [growl::Growl; OSCS],
    squeak: [squeak::Squeak; OSCS],
    tract: [tract::Tract; OSCS],
    symbio: [symbio::Symbio; OSCS],
    flora: [flora::Flora; OSCS],
    rumble: [rumble::Rumble; OSCS],
}

/// Run `$body` with `$o` bound to the selected engine's chamber array.
macro_rules! with_engine {
    ($self:ident, $engine:expr, $o:ident => $body:expr) => {
        match $engine {
            0 => {
                let $o = &mut $self.gurgle;
                $body
            }
            1 => {
                let $o = &mut $self.flatus;
                $body
            }
            2 => {
                let $o = &mut $self.growl;
                $body
            }
            3 => {
                let $o = &mut $self.squeak;
                $body
            }
            4 => {
                let $o = &mut $self.tract;
                $body
            }
            5 => {
                let $o = &mut $self.symbio;
                $body
            }
            6 => {
                let $o = &mut $self.flora;
                $body
            }
            _ => {
                let $o = &mut $self.rumble;
                $body
            }
        }
    };
}

#[inline]
fn mix<T: Osc>(oscs: &mut [T; OSCS], gains: &[f32; OSCS]) -> f32 {
    let mut y = 0.0;
    for (o, &g) in oscs.iter_mut().zip(gains) {
        // A silent chamber doesn't need to run: it picks up where it left
        // off (or where the trigger put it) once its envelope opens.
        if g != 0.0 {
            y += g * o.tick();
        }
    }
    y
}

impl Engines {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, engine: u32) {
        with_engine!(self, engine, o => o.iter_mut().for_each(Osc::reset));
    }

    pub fn control(&mut self, engine: u32, c: &Ctx, oscs: &[OscCtx; OSCS]) {
        with_engine!(self, engine, o => {
            for (osc, oc) in o.iter_mut().zip(oscs) {
                osc.control(c, oc);
            }
        });
    }

    pub fn trigger(&mut self, engine: u32, c: &Ctx, oscs: &[OscCtx; OSCS], rng: &mut Noise) {
        with_engine!(self, engine, o => {
            for (osc, oc) in o.iter_mut().zip(oscs) {
                osc.trigger(c, oc, rng);
            }
        });
    }

    /// One sample of the three chambers, each scaled by its gain
    /// (level × envelope × squeeze).
    #[inline]
    pub fn tick(&mut self, engine: u32, gains: &[f32; OSCS]) -> f32 {
        with_engine!(self, engine, o => mix(o, gains))
    }
}

/// Phase increment for `hz` at `fs`, kept below Nyquist.
#[inline]
pub(crate) fn inc(hz: f32, fs: f32) -> f32 {
    (hz / fs).clamp(0.0, 0.45)
}

/// Test helpers: run one chamber of an engine at fixed settings.
#[cfg(test)]
pub(crate) mod testing {
    use super::*;

    /// Render `seconds` of chamber 1 of `engine` at constant pressure `env`.
    pub fn run(
        engine: u32,
        fs: f32,
        hz: f32,
        xyz: (f32, f32, f32),
        env: f32,
        seconds: f32,
    ) -> Vec<f32> {
        run_with(engine, fs, hz, xyz, seconds, |_| env)
    }

    /// As [`run`], with the pressure a function of time in seconds.
    pub fn run_with(
        engine: u32,
        fs: f32,
        hz: f32,
        (x, y, z): (f32, f32, f32),
        seconds: f32,
        env: impl Fn(f32) -> f32,
    ) -> Vec<f32> {
        let mut c = Ctx::new(fs);
        (c.x, c.y, c.z) = (x, y, z);
        let mut o = OscCtx {
            hz,
            env: env(0.0),
            decay_s: 2.0,
        };
        let mut e = Engines::new();
        let oscs = [o; OSCS];
        e.trigger(engine, &c, &oscs, &mut Noise::new(7));
        let gains = [1.0, 0.0, 0.0];
        let n = (seconds * fs) as usize;
        (0..n)
            .map(|i| {
                if i % 16 == 0 {
                    o.env = env(i as f32 / fs);
                    e.control(engine, &c, &[o; OSCS]);
                }
                e.tick(engine, &gains)
            })
            .collect()
    }

    pub fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }

    pub fn peak(x: &[f32]) -> f32 {
        x.iter().fold(0.0f32, |a, &b| a.max(b.abs()))
    }

    /// Fundamental by autocorrelation, searched between `lo` and `hi` Hz:
    /// the shortest lag whose correlation is a local peak within 8% of the
    /// best one (so a period's multiples don't win by a hair).
    pub fn pitch(x: &[f32], fs: f32, lo: f32, hi: f32) -> f32 {
        let n = x.len() / 2;
        let r = |lag: usize| (0..n).map(|i| x[i] * x[i + lag]).sum::<f32>();
        let (l0, l1) = ((fs / hi) as usize, ((fs / lo) as usize).min(n - 1));
        let vals: Vec<f32> = (l0..=l1 + 1).map(r).collect();
        let best = vals[..vals.len() - 1]
            .iter()
            .cloned()
            .fold(f32::MIN, f32::max);
        let i = (0..vals.len() - 1)
            .find(|&i| {
                vals[i] >= 0.92 * best
                    && (i == 0 || vals[i] >= vals[i - 1])
                    && vals[i] >= vals[i + 1]
            })
            .unwrap_or(0);
        fs / (l0 + i) as f32
    }

    /// Normalized autocorrelation at the best lag in `lo..hi` Hz: near 1 for
    /// a steady periodic tone, low for irregular or noisy sound.
    pub fn periodicity(x: &[f32], fs: f32, lo: f32, hi: f32) -> f32 {
        let n = x.len() / 2;
        let r = |lag: usize| (0..n).map(|i| x[i] * x[i + lag]).sum::<f32>();
        let r0 = r(0).max(1e-12);
        let (l0, l1) = ((fs / hi) as usize, ((fs / lo) as usize).min(n - 1));
        (l0..=l1).map(r).fold(f32::MIN, f32::max) / r0
    }

    /// Spectral centroid via the first-difference proxy, in Hz.
    pub fn brightness(x: &[f32], fs: f32) -> f32 {
        let d = (x.windows(2).map(|p| (p[1] - p[0]).powi(2)).sum::<f32>() / x.len() as f32).sqrt();
        fs * d / (core::f32::consts::TAU * rms(x).max(1e-9))
    }

    /// RMS of consecutive `win`-sample windows.
    pub fn envelope(x: &[f32], win: usize) -> Vec<f32> {
        x.chunks_exact(win).map(rms).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use crate::params::{ENGINE_COUNT, ENGINE_FLORA, ENGINE_GURGLE};

    /// Every engine at every corner of its macros, at low and high pitch,
    /// full pressure: finite, audible and within a sane range.
    #[test]
    fn engines_are_bounded_and_audible_everywhere() {
        let fs = 96_000.0;
        for engine in 0..ENGINE_COUNT as u32 {
            for hz in [30.0f32, 250.0, 2_000.0] {
                for corner in 0..8u32 {
                    let m = |bit: u32| if corner & (1 << bit) != 0 { 1.0 } else { 0.0 };
                    let xyz = (m(0), m(1), m(2));
                    let out = run(engine, fs, hz, xyz, 1.0, 1.0);
                    assert!(
                        out.iter().all(|v| v.is_finite()),
                        "engine {engine} {hz} {xyz:?}"
                    );
                    let (p, r) = (peak(&out), rms(&out[48_000..]));
                    assert!(p < 4.0, "engine {engine} {hz} Hz {xyz:?}: peak {p}");
                    // Sparse bubbles and critters (DENSITY at zero) are meant
                    // to leave gaps; they only have to make *some* sound.
                    let sparse = matches!(engine, ENGINE_GURGLE | ENGINE_FLORA) && xyz.0 == 0.0;
                    if sparse {
                        assert!(p > 0.01, "engine {engine} {hz} Hz {xyz:?}: peak {p}");
                    } else {
                        assert!(r > 0.005, "engine {engine} {hz} Hz {xyz:?}: rms {r}");
                    }
                }
            }
        }
    }

    /// The driven engines fall silent by themselves when the pressure goes
    /// (no self-sustaining process leaks out after the envelope closes).
    #[test]
    fn driven_engines_settle_without_pressure() {
        use crate::params::*;
        let fs = 48_000.0;
        for engine in [
            ENGINE_GURGLE,
            ENGINE_FLATUS,
            ENGINE_SQUEAK,
            ENGINE_TRACT,
            ENGINE_FLORA,
        ] {
            let out = run_with(engine, fs, 120.0, (0.5, 0.5, 0.5), 3.0, |t| {
                if t < 0.5 { 1.0 } else { 0.0 }
            });
            let late = rms(&out[out.len() - 4_800..]);
            assert!(late < 0.05, "engine {engine}: {late}");
        }
    }
}
