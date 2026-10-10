//! GURGLE: gas bubbles detaching in digestive fluid.
//!
//! A bubble in a liquid rings at its Minnaert resonance, a damped sine whose
//! pitch is set by its size. As it detaches and rises, the pressure around it
//! changes and the pitch glides, which is what makes a bubble go "bloop"
//! rather than "ping" (K. van den Doel, "Physically-based models for liquid
//! sounds", ACM TAP 2(4), 2005). GURGLE releases a Poisson stream of them:
//!
//! * Each bubble's pitch is PITCH scattered log-normally (SCATTER), its
//!   damping follows van den Doel's fit `d = 0.13·f + 0.0072·f^1.5` with a
//!   random spread (small bubbles ring briefly, big ones wobble on), and its
//!   level is random.
//! * DENSITY is the release rate at full pressure. The rate follows the
//!   pressure, so the gas runs out as the envelope decays and a peristaltic
//!   squeeze pushes out a burst.
//! * CHIRP is how far each bubble glides over its life, bipolar: down to an
//!   octave (sinking), up to two (rising "plinks").
//!
//! Bubbles start at phase zero, so even a single one has a clean onset.

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{Noise, cos_turns, sin_turns};

const MAX_BUBBLES: usize = 16;
/// Fade-out time constant of a bubble making way for a new one.
const FADE_S: f32 = 0.004;
/// Re-evaluate each bubble's chirped frequency every this many samples.
const CHIRP_STRIDE: u32 = 8;

/// One bubble: a damped sine kept as a unit phasor `(re, im)` rotated by
/// `(rc, rs)` each sample (a complex multiply instead of a sine), with its
/// amplitude decaying alongside.
#[derive(Clone, Copy, Debug, Default)]
struct Bubble {
    amp: f32,
    amp0: f32,
    decay_mul: f32,
    re: f32,
    im: f32,
    rc: f32,
    rs: f32,
    inc0: f32,
}

impl Bubble {
    fn set_inc(&mut self, inc: f32) {
        self.rc = cos_turns(inc);
        self.rs = sin_turns(inc);
    }
}

#[derive(Clone, Debug, Default)]
pub struct Gurgle {
    rng: Noise,
    bubbles: [Bubble; MAX_BUBBLES],
    fs: f32,
    hz: f32,
    /// Probability of a release per sample.
    p_release: f32,
    scatter: f32,
    chirp: f32,
    velocity: f32,
    norm: f32,
    fade_mul: f32,
    count: u32,
}

impl Gurgle {
    fn release(&mut self) {
        let Some(slot) = self.bubbles.iter().position(|b| b.amp == 0.0) else {
            // Full: the quietest bubble fades out fast (no click) to make
            // room for the next one.
            let quietest = (0..MAX_BUBBLES)
                .min_by(|&a, &b| self.bubbles[a].amp.total_cmp(&self.bubbles[b].amp))
                .unwrap_or(0);
            self.bubbles[quietest].decay_mul = self.bubbles[quietest].decay_mul.min(self.fade_mul);
            return;
        };
        let rng = &mut self.rng;
        let f = (self.hz * (self.scatter * 0.6 * rng.gauss()).exp2()).clamp(8.0, 0.2 * self.fs);
        let d = (0.13 * f + 0.0072 * f.powf(1.5)) * (0.6 + 0.9 * rng.uniform());
        // Bigger (lower) bubbles than the centre come out a little louder.
        let size = (self.hz / f).sqrt().clamp(0.5, 2.0);
        let amp = (0.35 + 0.65 * rng.uniform()) * size * (0.5 + 0.5 * self.velocity);
        let inc0 = f / self.fs;
        let mut b = Bubble {
            amp,
            amp0: amp,
            decay_mul: (-d / self.fs).exp(),
            re: 1.0,
            im: 0.0,
            inc0,
            ..Bubble::default()
        };
        b.set_inc(inc0);
        self.bubbles[slot] = b;
    }
}

impl Osc for Gurgle {
    fn reset(&mut self) {
        self.bubbles = [Bubble::default(); MAX_BUBBLES];
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.control(c, o);
        // A hit always opens with a bubble.
        self.release();
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.fs = c.fs;
        self.hz = o.hz;
        self.velocity = c.velocity;
        let rate = map::gurgle_density(c.x) * o.env.max(0.0).sqrt();
        self.p_release = rate / c.fs;
        self.scatter = map::gurgle_scatter_octaves(c.y);
        self.chirp = map::gurgle_chirp_octaves(c.z);
        // Overlapping bubbles add up in power: keep a dense cloud near the
        // level of a sparse one. A bubble lives about 1/d seconds.
        let d = 0.13 * o.hz + 0.0072 * o.hz.powf(1.5);
        let overlap = (map::gurgle_density(c.x) / d.max(1.0)).min(MAX_BUBBLES as f32);
        self.norm = 1.3 / (1.0 + overlap).sqrt();
        self.fade_mul = (-1.0 / (FADE_S * c.fs)).exp();
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        if self.rng.uniform() < self.p_release {
            self.release();
        }
        self.count = self.count.wrapping_add(1);
        let stride = self.count.is_multiple_of(CHIRP_STRIDE);
        let rechirp = stride && self.chirp != 0.0;
        let mut y = 0.0;
        for b in self.bubbles.iter_mut() {
            if b.amp == 0.0 {
                continue;
            }
            y += b.amp * b.im;
            let (re, im) = (b.re, b.im);
            b.re = re * b.rc - im * b.rs;
            b.im = re * b.rs + im * b.rc;
            if stride {
                // Keep the phasor on the unit circle (one Newton step).
                let k = 1.5 - 0.5 * (b.re * b.re + b.im * b.im);
                b.re *= k;
                b.im *= k;
            }
            if rechirp {
                let life = 1.0 - b.amp / b.amp0;
                b.set_inc((b.inc0 * (self.chirp * life).exp2()).min(0.45));
            }
            b.amp *= b.decay_mul;
            if b.amp < 1e-4 {
                b.amp = 0.0;
            }
        }
        y * self.norm
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_GURGLE;

    /// Sparse: discrete bubbles with silence between. Dense: a steady cloud.
    #[test]
    fn density_goes_from_drips_to_a_boil() {
        let fs = 48_000.0;
        let gappy = |x: f32| {
            let out = run(ENGINE_GURGLE, fs, 300.0, (x, 0.3, 0.6), 1.0, 2.0);
            let env = envelope(&out, 480);
            let loud = env.iter().cloned().fold(0.0f32, f32::max);
            env.iter().filter(|&&e| e < 0.02 * loud).count() as f32 / env.len() as f32
        };
        assert!(gappy(0.15) > 0.4, "sparse {}", gappy(0.15));
        assert!(gappy(0.9) < 0.05, "dense {}", gappy(0.9));
    }

    /// The bubbles sit around PITCH.
    #[test]
    fn bubbles_ring_near_the_pitch() {
        let fs = 48_000.0;
        let out = run(ENGINE_GURGLE, fs, 400.0, (0.6, 0.0, 0.5), 1.0, 1.0);
        let b = brightness(&out, fs);
        assert!(b > 300.0 && b < 560.0, "{b}");
    }

    /// Rising bubbles end higher than they start.
    #[test]
    fn chirp_glides_up() {
        let fs = 48_000.0;
        let flat = run(ENGINE_GURGLE, fs, 200.0, (0.3, 0.0, 0.5), 1.0, 2.0);
        let up = run(ENGINE_GURGLE, fs, 200.0, (0.3, 0.0, 1.0), 1.0, 2.0);
        assert!(brightness(&up, fs) > 1.3 * brightness(&flat, fs));
    }
}
