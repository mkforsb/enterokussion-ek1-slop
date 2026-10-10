//! The modulators: peristalsis, the rhythmic squeeze of a gut, and chaos, a
//! slow Lorenz attractor used as an LFO that never repeats.

use crate::params::map;
use crate::util::{Noise, cos_turns};

/// Peristalsis: waves of contraction.
///
/// Each cycle is one squeeze, `c = ((1 − cos 2πφ)/2)²`: a rounded pulse with
/// slack time between (the gut relaxes longer than it contracts). With
/// IRREG up, every contraction gets its own length and strength, so the
/// rhythm stumbles like a real one. Each chamber hears the wave a little
/// later than the one before ([`Peristalsis::contraction`]'s lag), so a
/// squeeze rolls through the three chambers.
#[derive(Clone, Debug)]
pub struct Peristalsis {
    rng: Noise,
    phase: f32,
    /// This cycle's rate multiplier and strength.
    rate_mul: f32,
    strength: f32,
}

impl Peristalsis {
    /// Where a hit starts in the cycle: on the way into a squeeze.
    const START: f32 = 0.3;

    pub fn new(seed: u32) -> Self {
        Self {
            rng: Noise::new(seed ^ 0x5eed_9e75),
            phase: Self::START,
            rate_mul: 1.0,
            strength: 1.0,
        }
    }

    pub fn trigger(&mut self, irregularity: f32) {
        self.phase = Self::START;
        self.new_cycle(irregularity);
    }

    fn new_cycle(&mut self, irregularity: f32) {
        self.rate_mul = (0.45 * irregularity * self.rng.gauss()).exp2();
        self.strength = 1.0 - 0.8 * irregularity * self.rng.uniform();
    }

    /// Advance by `seconds` at `hz`.
    pub fn advance(&mut self, hz: f32, irregularity: f32, seconds: f32) {
        self.phase += hz * self.rate_mul * seconds;
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
            self.new_cycle(irregularity);
        }
    }

    /// Contraction `0..=1` heard `lag` cycles behind the wave front.
    pub fn contraction(&self, lag: f32) -> f32 {
        let p = self.phase - lag;
        let s = 0.5 - 0.5 * cos_turns(p - p.floor());
        s * s * self.strength
    }
}

const SIGMA: f32 = 10.0;
const RHO: f32 = 28.0;
const BETA: f32 = 8.0 / 3.0;
/// Lorenz time units per "cycle" (one loop around a lobe at ρ = 28).
const UNITS_PER_CYCLE: f32 = 0.62;
const MAX_STEP: f32 = 0.004;

#[derive(Clone, Debug)]
pub struct ChaosLfo {
    x: f32,
    y: f32,
    z: f32,
}

impl ChaosLfo {
    /// Different seeds start at different points on the attractor.
    pub fn new(seed: u32) -> Self {
        let mut c = Self {
            x: 1.0 + seed as f32 * 0.37,
            y: 1.0,
            z: 20.0,
        };
        // Settle onto the attractor.
        for _ in 0..4_000 {
            c.step(0.004);
        }
        c
    }

    fn step(&mut self, h: f32) {
        let d = |x: f32, y: f32, z: f32| (SIGMA * (y - x), x * (RHO - z) - y, x * y - BETA * z);
        let (dx, dy, dz) = d(self.x, self.y, self.z);
        let (dx, dy, dz) = d(
            self.x + 0.5 * h * dx,
            self.y + 0.5 * h * dy,
            self.z + 0.5 * h * dz,
        );
        self.x += h * dx;
        self.y += h * dy;
        self.z += h * dz;
    }

    /// Advance by `seconds` at the RATE knob position.
    pub fn advance(&mut self, rate: f32, seconds: f32) {
        let mut units = map::chaos_hz(rate) * UNITS_PER_CYCLE * seconds;
        while units > 0.0 {
            let h = units.min(MAX_STEP);
            self.step(h);
            units -= h;
        }
        if !(self.x.is_finite() && self.y.is_finite() && self.z.is_finite()) {
            *self = Self::new(0);
        }
    }

    /// Output in about `-1..=1`.
    pub fn value(&self) -> f32 {
        (self.x / 18.0).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peristalsis_pulses_at_its_rate_and_lags_per_chamber() {
        let mut p = Peristalsis::new(1);
        p.trigger(0.0);
        let dt = 0.001;
        let mut peaks = 0;
        let mut last = p.contraction(0.0);
        let mut rising = true;
        for _ in 0..10_000 {
            p.advance(2.0, 0.0, dt);
            let c = p.contraction(0.0);
            if rising && c < last {
                peaks += 1;
            }
            rising = c >= last;
            last = c;
        }
        // 10 s at 2 Hz.
        assert!((19..=21).contains(&peaks), "{peaks}");
        // A quarter-cycle lag: a different point of the wave.
        assert!((p.contraction(0.0) - p.contraction(0.25)).abs() > 0.05);
        assert!((0.0..=1.0).contains(&p.contraction(0.4)));
    }

    #[test]
    fn wanders_both_lobes_within_range() {
        let mut c = ChaosLfo::new(1);
        let (mut lo, mut hi) = (0.0f32, 0.0f32);
        for _ in 0..20_000 {
            c.advance(0.5, 0.001);
            lo = lo.min(c.value());
            hi = hi.max(c.value());
        }
        assert!(lo < -0.5 && hi > 0.5, "{lo} {hi}");
    }
}
