//! FLATUS: a pressure-driven sphincter valve, buzzing into a cheek cavity.
//!
//! The valve is a Van der Pol oscillator, the textbook self-excited
//! oscillator with a nonlinear damping term:
//!
//! ```text
//!  x'' − μ·(1 − x²)·x' + x = f(t)        (time in units of 1/ω)
//! ```
//!
//! Gas pressure behind the valve sets μ. Below a threshold μ goes negative
//! and the valve simply closes (a ring-down to silence, straight from the
//! equation). Just above it the valve wobbles almost sinusoidally (a soft
//! "pfff-ooo"), and the harder it is pushed, the more the motion turns into
//! a relaxation oscillation: long closed phases and snapping releases, the
//! tight "brrrap". Air only flows while the valve is open, so the output is
//! the flow through it, `max(0, x)²`, a train of puffs.
//!
//! What comes out is what recordings of the real thing show (measured on a
//! few, the "diseased gnome" among them): a low *thump* of moving air (the
//! flow itself), the *snap* of every opening (its derivative, which is what
//! a valve radiates), and above all *turbulence*: hiss and crackle chopped
//! up by the flapping valve. A clean buzz is the rare "trumpet" kind.
//!
//! * TIGHT: the valve's μ at full pressure. The relaxation lengthens the
//!   period, so the oscillator is retuned from a measured period table to
//!   stay on PITCH.
//! * WET: from a dry trumpet to a diseased sputter. Random forcing of the
//!   valve and a slow random wobble of μ, which can briefly stall it
//!   (irregular cycles, skipped puffs); and the output crossfades from the
//!   clean buzz towards turbulent hiss and wet crackle gated by the flow.
//!   At low PITCH (10–30 Hz of flapping) this is the full wet sputter.
//! * CHEEK: the cavity the puffs ring into, a resonance from 60 Hz to 2.4
//!   kHz: a muffled "bwomp" at the bottom, a nasal "pfrrt" at the top.
//!
//! Like a real one, the pitch sags as the pressure falls (up to about six
//! semitones at the end of the decay), and a peristaltic squeeze drives it
//! in and out of sputtering.

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{DcBlocker, Noise, SlowNoise, Svf, SvfCoeffs, fast_tanh};

/// Pressure below which the valve stays shut.
const THRESHOLD: f32 = 0.06;
/// Van der Pol period / 2π at μ = 0, 1, 2, … 6, 8 (computed with RK4).
const PERIOD: [f32; 8] = [1.0, 1.0604, 1.2143, 1.41, 1.6239, 1.8481, 2.0789, 2.5526];
/// Largest per-substep advance (radians of the oscillator) per unit of
/// stiffness.
const MAX_STEP: f32 = 0.1;

/// Period stretch of the relaxation oscillation at `mu`.
fn period_ratio(mu: f32) -> f32 {
    let mu = mu.max(0.0);
    if mu >= 8.0 {
        return PERIOD[7] + 0.237 * (mu - 8.0);
    }
    let (i, f) = if mu < 6.0 {
        let i = mu as usize;
        (i, mu - i as f32)
    } else {
        (6, (mu - 6.0) / 2.0)
    };
    PERIOD[i] + (PERIOD[i + 1] - PERIOD[i]) * f
}

#[derive(Clone, Debug, Default)]
pub struct Flatus {
    rng: Noise,
    x: f32,
    v: f32,
    /// Per-sample advance in oscillator radians, and the substeps it is cut into.
    step: f32,
    substeps: u32,
    mu: f32,
    wet: f32,
    /// Slow random wobbles of μ (stalls) and of the rate (uneven flaps).
    wobble: SlowNoise,
    drift: SlowNoise,
    force: f32,
    dc: DcBlocker,
    dc_r: f32,
    cheek: Svf,
    cheek_c: SvfCoeffs,
    /// Previous flow, for the snap (derivative).
    last_flow: f32,
    snap_scale: f32,
    /// Turbulence colouring: a band of roughly 300 Hz … 3 kHz.
    turb: Svf,
    turb_c: SvfCoeffs,
    crackle: f32,
    crackle_mul: f32,
    p_crackle: f32,
    norm: f32,
}

impl Osc for Flatus {
    fn reset(&mut self) {
        self.x = 0.0;
        self.v = 0.0;
        self.last_flow = 0.0;
        self.crackle = 0.0;
        self.turb.reset();
        self.dc.reset();
        self.cheek.reset();
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        // A resting valve needs a nudge; a buzzing one carries on.
        if self.x.abs() < 0.3 && self.v.abs() < 0.3 {
            self.x = 0.6 + 0.3 * self.rng.uniform();
            self.v = 0.0;
        }
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let fs = c.fs;
        let p = o.env.clamp(0.0, 1.0);
        self.wet = c.y;
        // Above the threshold μ rises with the pressure; below it, it goes
        // negative and the valve closes.
        let drive = (p - THRESHOLD) / (1.0 - THRESHOLD);
        let tight = map::flatus_mu(c.x) * (0.6 + 0.4 * c.velocity);
        // High valves can't be as stiff: keep the relaxation's edges in band.
        self.mu = tight * drive * (900.0 / o.hz.max(1.0)).min(1.0);
        let mu_typ = self.mu.max(0.0) * (1.0 + 0.5 * self.wet);
        // The pitch sags with the pressure, and the relaxation is retuned.
        let hz = o.hz * (0.7 + 0.3 * p);
        let step = core::f32::consts::TAU * hz / fs * period_ratio(mu_typ);
        let stiffness = 1.0 + 3.0 * mu_typ;
        self.substeps = ((step * stiffness / MAX_STEP).ceil() as u32).clamp(1, 32);
        self.step = (step / self.substeps as f32).min(0.5);
        self.wobble.set(6.0 + 0.3 * hz.min(100.0), fs);
        self.drift.set(4.0 + 0.2 * hz.min(100.0), fs);
        // Random kicks need gas behind them.
        self.force = 3.0 * self.wet * self.wet * p;
        self.dc_r = 1.0 - core::f32::consts::TAU * 15.0 / fs;
        self.cheek_c = SvfCoeffs::new(map::cheek_hz(c.z), 2.5, fs);
        // The flow's slope per sample scales with the pitch; undo that.
        self.snap_scale = 0.15 / (core::f32::consts::TAU * hz / fs).max(1e-5);
        // Wetter is darker and gloopier: the band slides down with WET. Faster
        // flow hisses higher (the Strouhal law), so it rises with pressure.
        self.turb_c = SvfCoeffs::new((1_800.0 - 700.0 * self.wet) * (0.45 + 0.55 * p), 0.6, fs);
        // Up to ~3000 pops a second at full flow.
        self.p_crackle = 3_000.0 * self.wet / fs;
        self.crackle_mul = (-1.0 / (0.000_25 * fs)).exp();
        // The turbulence carries more energy than the buzz; level them.
        self.norm = 1.0 / (1.0 + 1.4 * self.wet);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        // Sputter: μ wanders, now and then below zero for a few cycles (the
        // valve stalls, then has to build up again), and the flapping rate
        // stumbles.
        let wob = self.wobble.tick(&mut self.rng);
        let mu = self.mu * (1.0 + 1.6 * self.wet * wob);
        let h = self.step * (0.3 * self.wet * self.drift.tick(&mut self.rng)).exp2();
        let f = self.force * self.rng.sample();
        for _ in 0..self.substeps {
            // Semi-implicit Euler: velocity first, then position.
            self.v += h * (mu * (1.0 - self.x * self.x) * self.v - self.x + f);
            self.x += h * self.v;
        }
        if !(self.x.is_finite() && self.v.is_finite()) {
            self.reset();
        }
        self.x = self.x.clamp(-4.0, 4.0);
        self.v = self.v.clamp(-40.0, 40.0);
        let open = (0.5 * self.x).max(0.0);
        let flow = open * open;
        // A hard snap is still bounded: the valve can only open so fast.
        let snap = 1.2 * fast_tanh((flow - self.last_flow) * self.snap_scale / 1.2);
        self.last_flow = flow;
        let body = self.dc.process(flow, self.dc_r);
        let dry = 0.8 * body + snap;
        // Turbulence: highpassed noise and pops, both riding on the flow.
        let w = self.wet;
        let mut wet = 0.0;
        if w > 0.0 {
            if self.rng.uniform() < self.p_crackle * flow.min(1.0) {
                self.crackle = 3.0 * self.rng.sample();
            }
            self.crackle *= self.crackle_mul;
            let n = 1.7 * self.rng.sample() + self.crackle;
            let (_, band, _) = self.turb.process(flow.min(1.5) * n, &self.turb_c);
            wet = 2.0 * band * self.turb_c.k() + 0.5 * body;
        }
        let s = dry * (1.0 - 0.75 * w) + wet * 1.6 * w;
        let (_, bp, _) = self.cheek.process(s, &self.cheek_c);
        // The wildest corners can spike; round them off.
        let y = (0.6 * s + 0.9 * bp * self.cheek_c.k()) * self.norm;
        2.0 * fast_tanh(0.5 * y)
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use super::*;
    use crate::params::ENGINE_FLATUS;

    #[test]
    fn period_table_is_monotonic() {
        let mut last = 0.0;
        for i in 0..100 {
            let r = period_ratio(i as f32 * 0.1);
            assert!(r >= last);
            last = r;
        }
    }

    /// A dry valve buzzes on PITCH, soft or tight.
    #[test]
    fn dry_valve_buzzes_on_pitch() {
        let fs = 96_000.0;
        for tight in [0.2f32, 0.6, 1.0] {
            for hz in [60.0f32, 150.0, 400.0] {
                let out = run(ENGINE_FLATUS, fs, hz, (tight, 0.0, 0.5), 1.0, 1.0);
                let f = pitch(&out[48_000..], fs, hz * 0.5, hz * 1.6);
                let cents = 1200.0 * (f / hz).log2();
                assert!(cents.abs() < 80.0, "tight {tight} {hz} Hz: {f}");
            }
        }
    }

    /// TIGHT turns a round puff into a buzz.
    #[test]
    fn tight_is_brighter() {
        let fs = 96_000.0;
        let b = |t: f32| {
            brightness(
                &run(ENGINE_FLATUS, fs, 90.0, (t, 0.0, 0.2), 1.0, 1.0)[24_000..],
                fs,
            )
        };
        assert!(b(1.0) > 1.5 * b(0.0), "{} {}", b(1.0), b(0.0));
    }

    /// WET breaks the regular buzz into an irregular sputter.
    #[test]
    fn wet_sputters() {
        let fs = 48_000.0;
        let reg = |w: f32| {
            let out = run(ENGINE_FLATUS, fs, 80.0, (0.6, w, 0.3), 1.0, 2.0);
            periodicity(&out[24_000..], fs, 40.0, 160.0)
        };
        assert!(reg(0.0) > 0.8, "dry {}", reg(0.0));
        assert!(
            reg(1.0) < 0.75 * reg(0.0),
            "wet {} dry {}",
            reg(1.0),
            reg(0.0)
        );
    }

    /// Below the threshold the valve shuts.
    #[test]
    fn no_pressure_no_buzz() {
        let fs = 48_000.0;
        let out = run(ENGINE_FLATUS, fs, 100.0, (0.5, 0.0, 0.5), 0.03, 2.0);
        assert!(rms(&out[72_000..]) < 1e-3, "{}", rms(&out[72_000..]));
    }

    /// The pitch sags as the pressure falls.
    #[test]
    fn pitch_sags_with_pressure() {
        let fs = 48_000.0;
        let hi = run(ENGINE_FLATUS, fs, 120.0, (0.4, 0.0, 0.5), 1.0, 1.0);
        let lo = run(ENGINE_FLATUS, fs, 120.0, (0.4, 0.0, 0.5), 0.3, 1.0);
        let (fh, fl) = (
            pitch(&hi[24_000..], fs, 50.0, 200.0),
            pitch(&lo[24_000..], fs, 50.0, 200.0),
        );
        assert!(fl < fh * 0.9, "{fh} {fl}");
    }
}
