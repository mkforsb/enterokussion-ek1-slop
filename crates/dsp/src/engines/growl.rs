//! GROWL: a creaky pulse source through a morphing set of formants.
//!
//! A stomach growl, like a voice, is a train of pulses from something that
//! opens and snaps shut, coloured by the cavities behind it. The source here
//! is a Rosenberg-style flow pulse (a raised-cosine opening, a quarter-cosine
//! closing that ends abruptly, then a closed phase); its derivative excites
//! three resonant formant filters.
//!
//! * VOWEL morphs the formants through OO, OH, AH, EH and EE.
//! * ROUGH is what turns a vowel into a growl. It adds cycle-to-cycle period
//!   jitter, amplitude shimmer and breath noise during the open phase, and
//!   above a third of its travel *period doubling*: every other pulse comes
//!   weaker and later, so a subharmonic an octave down rattles under the
//!   tone (the "creak" of vocal fry, the rattle of a beast's throat).
//! * SIZE scales all formants from ×0.35 (something enormous, the formants
//!   squeezed down onto the fundamental) to ×1.6 (a small, nasal critter).
//!
//! Formant data: G. E. Peterson and H. L. Barney, "Control methods used in
//! a study of the vowels", JASA 24(2), 1952 (rounded).

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{DcBlocker, Noise, Svf, SvfCoeffs, fast_tanh};

/// F1–F3 of OO, OH, AH, EH, EE.
const VOWELS: [[f32; 3]; 5] = [
    [300.0, 870.0, 2240.0],
    [500.0, 900.0, 2600.0],
    [730.0, 1090.0, 2440.0],
    [530.0, 1840.0, 2480.0],
    [270.0, 2290.0, 3010.0],
];
/// Formant bandwidths at SIZE ×1, Hz.
const BANDWIDTHS: [f32; 3] = [90.0, 120.0, 180.0];
/// Formant gains. The resonators are used with their natural gain (≈ Q at
/// the centre), like the cavities they stand for, so narrow high formants
/// ring out of the source's falling spectrum.
const FORMANT_GAINS: [f32; 3] = [0.5, 0.45, 0.35];
/// Opening and closing durations as fractions of the period.
const OPEN: f32 = 0.36;
const CLOSE: f32 = 0.18;

/// Glottal flow over one period, `0..=1`.
#[inline]
fn flow(phase: f32) -> f32 {
    if phase < OPEN {
        0.5 - 0.5 * crate::util::cos_turns(0.5 * phase / OPEN)
    } else if phase < OPEN + CLOSE {
        crate::util::cos_turns(0.25 * (phase - OPEN) / CLOSE)
    } else {
        0.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Growl {
    rng: Noise,
    phase: f32,
    inc: f32,
    /// This cycle's period stretch and amplitude.
    cycle_mul: f32,
    cycle_amp: f32,
    odd: bool,
    last_flow: f32,
    jitter: f32,
    shimmer: f32,
    sub: f32,
    breath: f32,
    formants: [Svf; 3],
    coeffs: [SvfCoeffs; 3],
    dc: DcBlocker,
    dc_r: f32,
    /// Excitation scale: the derivative of the flow is ∝ the pitch.
    scale: f32,
    norm: f32,
}

impl Growl {
    fn new_cycle(&mut self) {
        let rng = &mut self.rng;
        self.odd = !self.odd;
        let mut mul = 1.0 + self.jitter * rng.gauss();
        let mut amp = 1.0 - self.shimmer * rng.uniform();
        if self.odd {
            mul *= 1.0 + 0.3 * self.sub;
            amp *= 1.0 - 0.75 * self.sub;
        }
        self.cycle_mul = 1.0 / mul.clamp(0.5, 2.0);
        self.cycle_amp = amp;
    }
}

impl Osc for Growl {
    fn reset(&mut self) {
        self.phase = 0.0;
        self.last_flow = 0.0;
        self.formants = [Svf::default(); 3];
        self.dc.reset();
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.reset();
        self.control(c, o);
        self.new_cycle();
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let fs = c.fs;
        self.inc = (o.hz / fs).min(0.25);
        let r = c.y;
        self.jitter = 0.1 * r * r;
        self.shimmer = 0.5 * r;
        self.sub = ((r - 0.33) / 0.67).clamp(0.0, 1.0);
        self.breath = 0.12 * r;
        let (i, f) = map::segment(c.x, VOWELS.len());
        let size = map::growl_size(c.z);
        for k in 0..3 {
            let hz = (VOWELS[i][k] + (VOWELS[i + 1][k] - VOWELS[i][k]) * f) * size;
            let bw = BANDWIDTHS[k] * size.sqrt();
            self.coeffs[k] = SvfCoeffs::new(hz, (hz / bw).max(0.7), fs);
        }
        self.dc_r = 1.0 - core::f32::consts::TAU * 15.0 / fs;
        // A unit-height flow pulse has slopes ~π/(2·CLOSE) per period.
        self.scale = 0.35 / (self.inc * core::f32::consts::PI / (2.0 * CLOSE)).max(1e-6);
        // A big creature's formants crowd the fundamental and ring louder.
        self.norm = 3.5 * size.sqrt();
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        self.phase += self.inc * self.cycle_mul;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.new_cycle();
        }
        let g = flow(self.phase) * self.cycle_amp;
        let mut e = (g - self.last_flow) * self.scale;
        self.last_flow = g;
        // Breath: turbulence while the valve is open.
        e += self.breath * g * self.rng.sample();
        let mut y = 0.1 * self.dc.process(g, self.dc_r);
        for k in 0..3 {
            let (_, bp, _) = self.formants[k].process(e, &self.coeffs[k]);
            y += FORMANT_GAINS[k] * bp;
        }
        // Huge, rough and low can ring hard; round the peaks off.
        let y = y * self.norm;
        2.0 * fast_tanh(0.5 * y)
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_GROWL;

    #[test]
    fn smooth_growl_is_on_pitch() {
        let fs = 96_000.0;
        for hz in [55.0f32, 110.0, 220.0] {
            let out = run(ENGINE_GROWL, fs, hz, (0.5, 0.0, 0.5), 1.0, 0.5);
            let f = pitch(&out[9_600..], fs, hz * 0.7, hz * 1.4);
            assert!((f / hz - 1.0).abs() < 0.03, "{hz}: {f}");
        }
    }

    /// Full ROUGH puts a subharmonic under the tone: the best period is
    /// twice the clean one.
    #[test]
    fn rough_rattles_an_octave_down() {
        let fs = 48_000.0;
        let out = run(ENGINE_GROWL, fs, 120.0, (0.3, 1.0, 0.5), 1.0, 1.0);
        let f = pitch(&out[4_800..], fs, 40.0, 200.0);
        assert!((f / 60.0 - 1.0).abs() < 0.15, "{f}");
    }

    /// EE is brighter than OO.
    #[test]
    fn vowels_move_the_formants() {
        let fs = 48_000.0;
        let b = |x: f32| brightness(&run(ENGINE_GROWL, fs, 100.0, (x, 0.0, 0.6), 1.0, 0.5), fs);
        assert!(b(1.0) > 1.4 * b(0.0), "{} {}", b(1.0), b(0.0));
    }
}
