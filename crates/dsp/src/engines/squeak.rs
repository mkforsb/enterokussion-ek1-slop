//! SQUEAK: stick-slip friction rubbing a wet, rubbery membrane.
//!
//! Something slides across a membrane under pressure. While the friction
//! holds, the membrane is dragged along; when it lets go, it springs back,
//! and the friction grips again. Rubber on glass, a wet finger on a balloon,
//! two organs rubbing in a cramped abdomen. The model is the classic bowed
//! friction loop (as in STK's `Bowed`), but rubbing three modes of a
//! membrane instead of a string:
//!
//! ```text
//!  Δv = v_slide − Σ wₖ·vₖ       F = Δv · (|s·Δv| + 0.75)⁻⁴       mode k ← F
//! ```
//!
//! The friction force falls as the slip speed rises (the negative slope that
//! feeds the oscillation) and stays bounded, so however hard it is pushed the
//! modes, being stable resonators, can't run away. The curve's steepest
//! negative slope is about −0.25 whatever its scale, so what decides whether
//! (and how wildly) it squeaks is the loop gain against it and where on the
//! curve the slide speed sits.
//!
//! * PRESS: contact pressure, the loop gain. Light pressure barely gets the
//!   membrane going: a pure, whistling squeak on the first mode. Heavy
//!   pressure grips hard and lets go violently: a raucous, buzzing creak in
//!   which the upper modes join in and fight.
//! * SLIDE: where the slip speed sits on the friction curve. Low, it only
//!   just slips; high, it skates over the top and thins out. It follows the
//!   pressure, so a swell squeaks into life and a decay sputters out.
//! * MEMBR: the mode ratios, from a tube (1 : 2 : 3) through a membrane
//!   (1 : 1.59 : 2.14) to gristle (1 : 2.76 : 5.40, a free bar).
//!
//! The modes are complex one-pole resonators, which stay stable under any
//! pitch sweep.

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{DcBlocker, Noise, cos_turns, flush, sin_turns};

const MODES: usize = 3;
/// Mode ratios of a tube, a membrane (Bessel zeros) and a free bar.
const RATIOS: [[f32; MODES]; 3] = [[1.0, 2.0, 3.0], [1.0, 1.594, 2.136], [1.0, 2.757, 5.404]];
/// How strongly each mode moves at the contact point.
const WEIGHTS: [f32; MODES] = [1.0, 0.6, 0.4];
const Q: f32 = 70.0;
/// Scale of the friction curve (irrelevant to the sound, the output is
/// normalized by the slide speed).
const SLOPE: f32 = 4.0;

#[derive(Clone, Debug, Default)]
pub struct Squeak {
    rng: Noise,
    re: [f32; MODES],
    im: [f32; MODES],
    /// Pole of each mode: r·e^{iθ}.
    pr: [f32; MODES],
    pi: [f32; MODES],
    inject: [f32; MODES],
    slide: f32,
    loop_gain: f32,
    norm: f32,
    dc: DcBlocker,
    dc_r: f32,
}

impl Osc for Squeak {
    fn reset(&mut self) {
        self.re = [0.0; MODES];
        self.im = [0.0; MODES];
        self.dc.reset();
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.control(c, o);
        // The first slip: a nudge so the squeak doesn't have to grow from
        // the contact noise alone.
        for k in 0..MODES {
            self.re[k] += 0.3 * self.slide * self.rng.sample();
        }
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let fs = c.fs;
        let (i, f) = map::segment(c.z, RATIOS.len());
        for k in 0..MODES {
            let ratio = RATIOS[i][k] * (RATIOS[i + 1][k] / RATIOS[i][k]).powf(f);
            let hz = (o.hz * ratio).min(0.45 * fs);
            let r = (-core::f32::consts::PI * hz / (Q * fs)).exp();
            let th = hz / fs;
            self.pr[k] = r * cos_turns(th);
            self.pi[k] = r * sin_turns(th);
            self.inject[k] = (1.0 - r) * WEIGHTS[k];
        }
        self.loop_gain = (16.0 + 60.0 * c.x * c.x) * (0.7 + 0.3 * c.velocity);
        let p = o.env.clamp(0.0, 1.0);
        // Position on the friction curve (in units of 1/SLOPE).
        // With no pressure at all, nothing slides.
        let a = (0.35 + 0.6 * c.y) * (0.4 + 0.6 * p.sqrt()) * (p / 0.05).min(1.0);
        let top = (0.35 + 0.6 * c.y) / SLOPE;
        self.slide = a / SLOPE;
        self.dc_r = 1.0 - core::f32::consts::TAU * 20.0 / fs;
        self.norm = 0.5 / top;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut v = 0.0;
        for k in 0..MODES {
            v += WEIGHTS[k] * self.re[k];
        }
        // A little roughness in the contact gets the slipping going.
        let dv = self.slide - v + 0.002 * self.rng.sample();
        let t = ((dv * SLOPE).abs() + 0.75).powi(-4).min(1.0);
        let force = dv * t * self.loop_gain;
        for k in 0..MODES {
            let (re, im) = (self.re[k], self.im[k]);
            self.re[k] = flush(re * self.pr[k] - im * self.pi[k] + self.inject[k] * force);
            self.im[k] = flush(re * self.pi[k] + im * self.pr[k]);
        }
        if !v.is_finite() {
            self.reset();
            return 0.0;
        }
        // The membrane's motion, plus some of the jerky stick-slip force
        // itself: the squeak's edge. While it sticks, the membrane is simply
        // dragged along: that offset isn't sound.
        self.dc.process(v + 0.08 * force, self.dc_r) * self.norm
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_SQUEAK;

    /// Light pressure, a decent slide: a periodic squeak at the pitch.
    #[test]
    fn squeaks_on_pitch() {
        let fs = 96_000.0;
        for hz in [200.0f32, 600.0, 1_500.0] {
            let out = run(ENGINE_SQUEAK, fs, hz, (0.3, 0.7, 0.5), 1.0, 1.0);
            let tail = &out[48_000..];
            assert!(rms(tail) > 0.05, "{hz}: {}", rms(tail));
            let f = pitch(tail, fs, hz * 0.7, hz * 1.4);
            assert!((f / hz - 1.0).abs() < 0.06, "{hz}: {f}");
        }
    }
}
