//! FIZZ: the acid layer under the chambers. Hiss, sizzle and crackle.
//!
//! White noise and a stream of tiny pops (random impulses, each a fraction
//! of a millisecond long) are mixed by CRACKLE and passed through a
//! resonant band around TONE. At zero CRACKLE it is a smooth hiss; turning
//! it up thins the noise out into effervescence (hundreds of pops a
//! second), then into the sparse crackle of something dissolving. The pops
//! get louder as they get fewer, so the layer keeps its level.
//!
//! The voice scales it by chamber 1's envelope and peristaltic squeeze.

use crate::params::map;
use crate::util::{Noise, Svf, SvfCoeffs};

/// Decay time constant of one pop.
const POP_TAU_S: f32 = 0.000_3;

#[derive(Clone, Debug)]
pub struct Fizz {
    band: Svf,
    coeffs: SvfCoeffs,
    hiss: f32,
    pops: f32,
    p_pop: f32,
    pop: f32,
    pop_mul: f32,
    pop_gain: f32,
}

impl Fizz {
    pub fn new(fs: f32) -> Self {
        Self {
            band: Svf::default(),
            coeffs: SvfCoeffs::new(2_000.0, 1.2, fs),
            hiss: 1.0,
            pops: 0.0,
            p_pop: 0.0,
            pop: 0.0,
            pop_mul: (-1.0 / (POP_TAU_S * fs)).exp(),
            pop_gain: 1.0,
        }
    }

    /// Knob positions; once per control block.
    pub fn update(&mut self, tone: f32, crackle: f32, fs: f32) {
        self.coeffs = SvfCoeffs::new(map::fizz_hz(tone), 1.2, fs);
        self.hiss = (1.0 - crackle).powf(1.5);
        self.pops = crackle.sqrt();
        let rate = 3_000.0 * 0.005f32.powf(crackle);
        self.p_pop = rate / fs;
        // Each pop carries ~τ·fs/2 of energy; keep the stream near unit power.
        self.pop_gain = (2.0 / (rate * POP_TAU_S)).sqrt().min(60.0);
        self.pop_mul = (-1.0 / (POP_TAU_S * fs)).exp();
    }

    #[inline]
    pub fn tick(&mut self, rng: &mut Noise) -> f32 {
        let mut x = self.hiss * rng.sample() * 1.7;
        if self.pops > 0.0 {
            if rng.uniform() < self.p_pop {
                self.pop = rng.sample();
            }
            x += self.pops * self.pop * self.pop_gain;
            self.pop *= self.pop_mul;
        }
        let (_, bp, _) = self.band.process(x, &self.coeffs);
        bp * self.coeffs.k()
    }

    pub fn reset(&mut self) {
        self.band.reset();
        self.pop = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(crackle: f32) -> Vec<f32> {
        let fs = 48_000.0;
        let mut f = Fizz::new(fs);
        f.update(0.5, crackle, fs);
        let mut rng = Noise::new(3);
        (0..48_000).map(|_| f.tick(&mut rng)).collect()
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
    }

    /// Smooth hiss and sparse crackle sit within a few dB of each other.
    #[test]
    fn crackle_keeps_its_level() {
        let (hiss, crackle) = (rms(&run(0.0)), rms(&run(1.0)));
        let db = 20.0 * (crackle / hiss).log10();
        assert!(db.abs() < 6.0, "{db}");
    }

    /// Crackle is spiky: a high crest factor.
    #[test]
    fn crackle_is_spiky() {
        let crest = |x: &[f32]| x.iter().fold(0.0f32, |a, &b| a.max(b.abs())) / rms(x);
        assert!(crest(&run(1.0)) > 2.0 * crest(&run(0.0)));
    }
}
