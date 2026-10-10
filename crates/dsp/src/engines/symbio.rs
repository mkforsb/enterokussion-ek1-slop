//! SYMBIO: two symbiotic organisms phase-modulating each other.
//!
//! Inherited from Xenokussion's FLUX. Two sine "organisms" each bend the
//! other's phase, and the host also feeds back on itself:
//!
//! ```text
//!  partner = sin(φ₂ + m·host[n−1])
//!  host    = sin(φ₁ + m·partner + f·host[n−1])
//! ```
//!
//! One-way, that would be plain FM. Both ways, it is a nonlinear loop: at
//! low coupling the pair hums along together, near whole-number ratios they
//! lock, and past a point they stop settling into any periodic orbit and the
//! tone dissolves into a seething, chaotic churn.
//!
//! The coupling follows the pressure (and velocity), so a swell starts as
//! two calm tones and boils over at its peak, and a peristaltic squeeze
//! stirs the pair up on every contraction. Unlike FLUX the coupling doesn't
//! decay on its own: these organisms live as long as the envelope does.
//!
//! * RATIO: the partner's frequency ratio, 0.5 … 7.13.
//! * COUPLE: peak coupling index.
//! * FEEDBK: the host's self-feedback, sine → saw → noise.

use super::{Ctx, Osc, OscCtx, inc};
use crate::params::map;
use crate::util::{Noise, sin_turns};

#[derive(Clone, Debug, Default)]
pub struct Symbio {
    p1: f32,
    p2: f32,
    y1: f32,
    y2: f32,
    inc1: f32,
    inc2: f32,
    /// Coupling in turns.
    index: f32,
    feedback: f32,
}

impl Osc for Symbio {
    fn reset(&mut self) {
        self.p1 = 0.0;
        self.p2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, _rng: &mut Noise) {
        self.reset();
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.inc1 = inc(o.hz, c.fs);
        self.inc2 = inc(o.hz * map::symbio_ratio(c.x), c.fs);
        let drive = (0.25 + 0.75 * o.env.clamp(0.0, 1.0)) * (0.5 + 0.5 * c.velocity);
        self.index = map::symbio_index(c.y) * drive / core::f32::consts::TAU;
        self.feedback = 0.3 * c.z * c.z;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let m = self.index;
        let y2 = sin_turns(self.p2 + m * self.y1);
        let y1 = sin_turns(self.p1 + m * y2 + self.feedback * self.y1);
        self.y1 = y1;
        self.y2 = y2;
        self.p1 += self.inc1;
        if self.p1 >= 1.0 {
            self.p1 -= 1.0;
        }
        self.p2 += self.inc2;
        if self.p2 >= 1.0 {
            self.p2 -= 1.0;
        }
        0.45 * y1 + 0.15 * y2
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_SYMBIO;

    /// Calm pair: periodic. Strongly coupled at an awkward ratio: chaos.
    #[test]
    fn coupling_breaks_into_chaos() {
        let fs = 48_000.0;
        let calm = run(ENGINE_SYMBIO, fs, 110.0, (0.37, 0.1, 0.0), 1.0, 1.0);
        let wild = run(ENGINE_SYMBIO, fs, 110.0, (0.37, 1.0, 0.5), 1.0, 1.0);
        let (pc, pw) = (
            periodicity(&calm[4_800..], fs, 30.0, 400.0),
            periodicity(&wild[4_800..], fs, 30.0, 400.0),
        );
        assert!(pc > 0.6 && pw < 0.6 * pc, "{pc} {pw}");
    }
}
