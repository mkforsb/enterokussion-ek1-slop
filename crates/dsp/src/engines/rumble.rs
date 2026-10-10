//! RUMBLE: borborygmus, the deep rolling rumble of a hungry gut.
//!
//! Three low partials (the fundamental, an octave-ish and a twelfth-ish) each
//! drift in pitch on their own slow random walk, so they beat against each
//! other and never quite agree. Gas pushing through roughens the sound: a
//! noisy amplitude modulation in the 10–60 Hz range (the "brrr"), and a
//! breathy band of noise around the third harmonic. A biased `tanh`
//! shaper adds the grit.
//!
//! * WANDER: depth (and speed) of the partials' pitch drift, up to ±12
//!   semitones: from a steady drone to a seasick moan.
//! * ROUGH: the noisy amplitude modulation and the gas noise.
//! * GRIT: drive into an asymmetric shaper (even and odd harmonics), up to
//!   +24 dB: from a round hum to a snarling grumble.

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{Noise, SlowNoise, Svf, SvfCoeffs, fast_tanh, one_pole_coeff, sin_turns};

const PARTIALS: usize = 3;
/// Slightly stretched harmonics: they beat slowly even before they wander.
const RATIOS: [f32; PARTIALS] = [1.0, 2.013, 2.996];
const AMPS: [f32; PARTIALS] = [1.0, 0.4, 0.25];

#[derive(Clone, Debug, Default)]
pub struct Rumble {
    rng: Noise,
    phase: [f32; PARTIALS],
    inc: [f32; PARTIALS],
    /// Each partial's drift (random walk, in −1 … 1 units).
    drift: [f32; PARTIALS],
    rough: f32,
    am: SlowNoise,
    gas: Svf,
    gas_c: SvfCoeffs,
    drive: f32,
    bias: f32,
    offset: f32,
    shaper_norm: f32,
}

impl Osc for Rumble {
    fn reset(&mut self) {
        self.phase = [0.0; PARTIALS];
        self.gas.reset();
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.reset();
        for d in self.drift.iter_mut() {
            *d = 0.5 * self.rng.sample();
        }
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let fs = c.fs;
        let block_s = 16.0 / fs;
        let wander = map::rumble_wander_semitones(c.x);
        // Random walk at a rate that rises with the depth: 0.3 … 3 Hz.
        let k = one_pole_coeff(
            1.0 / (core::f32::consts::TAU * (0.3 + 2.7 * c.x)),
            1.0 / block_s,
        );
        for i in 0..PARTIALS {
            self.drift[i] += (1.7 * self.rng.gauss() - self.drift[i]) * k;
            let semis = wander * self.drift[i].clamp(-1.5, 1.5);
            self.inc[i] = (o.hz * RATIOS[i] * (semis / 12.0).exp2() / fs).min(0.45);
        }
        self.rough = c.y;
        self.am.set(10.0 + 50.0 * c.y, fs);
        self.gas_c = SvfCoeffs::new(3.0 * o.hz, 2.0, fs);
        self.drive = (map::rumble_grit_db(c.z) / 20.0 * core::f32::consts::LN_10).exp();
        self.bias = 0.35 * c.z;
        self.offset = fast_tanh(self.drive * self.bias);
        let hi = fast_tanh(self.drive * (1.0 + self.bias)) - self.offset;
        let lo = self.offset - fast_tanh(self.drive * (self.bias - 1.0));
        self.shaper_norm = 1.0 / hi.max(lo).max(1e-3);
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut s = 0.0;
        for i in 0..PARTIALS {
            s += AMPS[i] * sin_turns(self.phase[i]);
            self.phase[i] += self.inc[i];
            if self.phase[i] >= 1.0 {
                self.phase[i] -= 1.0;
            }
        }
        s *= 1.0 / 1.65;
        let am = (1.0 + 0.9 * self.rough * self.am.tick(&mut self.rng)).clamp(0.0, 2.5);
        let n = self.rng.sample();
        let (_, gas, _) = self.gas.process(n, &self.gas_c);
        let s = s * am + 0.35 * self.rough * gas * self.gas_c.k() * am;
        (fast_tanh(self.drive * (s + self.bias)) - self.offset) * self.shaper_norm * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_RUMBLE;

    /// Calm: a steady hum at PITCH. Rough: the level churns.
    #[test]
    fn rough_churns_the_level() {
        let fs = 48_000.0;
        let churn = |y: f32| {
            let out = run(ENGINE_RUMBLE, fs, 60.0, (0.0, y, 0.0), 1.0, 3.0);
            let env = envelope(&out, 1_200);
            let mean = env.iter().sum::<f32>() / env.len() as f32;
            (env.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / env.len() as f32).sqrt() / mean
        };
        assert!(
            churn(1.0) > 4.0 * churn(0.0).max(0.02),
            "{} {}",
            churn(1.0),
            churn(0.0)
        );
    }

    #[test]
    fn calm_rumble_is_on_pitch() {
        let fs = 48_000.0;
        let out = run(ENGINE_RUMBLE, fs, 70.0, (0.0, 0.0, 0.0), 1.0, 1.0);
        let f = pitch(&out[4_800..], fs, 30.0, 100.0);
        assert!((f / 70.0 - 1.0).abs() < 0.08, "{f}");
    }
}
